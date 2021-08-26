package handler

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"path"
	"path/filepath"
	"plugin"
	plugins "skynet/plugin"
	"skynet/sn"
	"skynet/sn/utils"
	"strings"

	"github.com/google/uuid"
	log "github.com/sirupsen/logrus"
)

// PluginLoad is struct for loaded plugin
type PluginLoad struct {
	*plugins.PluginInstance
	Enable    bool                    // is plugin enabled
	Message   string                  // plugin message
	Interface plugins.PluginInterface `json:"-"` // plugin interface
	Loader    *plugin.Plugin          `json:"-"` // golang plugin loader
}

func (p *PluginLoad) Disable(msg string) {
	p.Enable = false
	p.Message = msg
}

var (
	PluginNotFoundError    = errors.New("Plugin not found")
	PluginIDDuplicateError = errors.New("Plugin ID duplicated")
	PluginInvalidError     = errors.New("Plugin invalid")
	PluginExistsError      = errors.New("Plugin already exists")
)

type sitePlugin struct {
	plugin PluginMap // plugin map
}

func (p *sitePlugin) readPlugin(path string) error {
	pPlugin, err := plugin.Open(path)
	if err != nil {
		return err
	}
	pSymbol, err := pPlugin.Lookup("NewPlugin")
	if err != nil {
		return err
	}
	pInterface := pSymbol.(func() plugins.PluginInterface)()
	pInstance := pInterface.Instance()
	pInstance.Path = filepath.Dir(path)
	if v, ok := p.plugin.Get(pInstance.ID); ok {
		return fmt.Errorf("%w: %v and %v have same ID %v", PluginIDDuplicateError, v.Name, pInstance.Name, v.ID)
	}
	p.plugin.Set(pInstance.ID, &PluginLoad{
		PluginInstance: pInstance,
		Enable:         false,
		Interface:      pInterface,
		Loader:         pPlugin,
	})
	log.WithFields(log.Fields{
		"id":      pInstance.ID,
		"name":    pInstance.Name,
		"version": pInstance.Version,
	}).Info("Plugin loaded")
	return nil
}

func (p *sitePlugin) readPluginFolder(dir string) {
	dirFile, err := ioutil.ReadDir(dir)
	if err != nil {
		log.Error(err)
		return
	}
	for _, df := range dirFile {
		if strings.HasSuffix(df.Name(), ".so") {
			soFile := dir + "/" + df.Name()
			if err := p.readPlugin(soFile); err != nil {
				log.Error(err)
			}
		}
	}
}

func (p *sitePlugin) cleanPlugin() {
	setting := sn.Skynet.Setting.GetCache()
	// setting enable cleanup
	for k, v := range setting {
		if strings.HasPrefix(k, "plugin_") && v == "1" {
			setting[k] = "-1"
		}
	}

	p.plugin.Range(func(k uuid.UUID, v *PluginLoad) bool {
		name := fmt.Sprintf("plugin_%s", v.ID.String())
		if status, exist := setting[name]; exist {
			v.Enable = status == "-1"
			if v.Enable {
				setting[name] = "1"
			}
		} else {
			if err := sn.Skynet.Setting.New(name, "0"); err != nil {
				log.Error(err)
			}
		}
		return true
	})

	for k, v := range setting {
		if strings.HasPrefix(k, "plugin_") && v == "-1" {
			if err := sn.Skynet.Setting.Delete(k); err != nil {
				log.Error(err)
			}
		}
	}
}

func (p *sitePlugin) checkPlugin(v *PluginLoad) bool {
	// check version
	c, err := utils.CheckSkynetVersion(v.SkynetVersion)
	if err != nil {
		log.Errorf("%w: Version constraint %v invalid (%v)", PluginInvalidError, v.SkynetVersion, err.Error())
	}
	if !c {
		v.Disable(fmt.Sprintf("Skynet version mismatch, need %s", v.SkynetVersion))
		log.Errorf("Plugin %v need skynet version %v, disable now.", v.Name, v.SkynetVersion)
		if err := sn.Skynet.Setting.Update("plugin_"+v.ID.String(), "0"); err != nil {
			log.Error(err)
		}
		return false
	}

	return true
}

func NewPlugin(base string) (sn.SNPlugin, error) {
	var ret sitePlugin

	files, err := ioutil.ReadDir(base)
	if err != nil {
		return nil, err
	}
	for _, f := range files {
		if f.IsDir() {
			ret.readPluginFolder(path.Join(base, f.Name()))
		}
	}
	ret.cleanPlugin()
	ret.plugin.Range(func(k uuid.UUID, v *PluginLoad) bool {
		ret.checkPlugin(v)
		return true
	})

	// plugin init
	ret.plugin.Range(func(k uuid.UUID, v *PluginLoad) bool {
		if v.Enable {
			if err := v.Interface.PluginInit(); err != nil {
				v.Disable(fmt.Sprintf("PluginInit error: %s", err.Error()))
			}
		}
		return true
	})
	return &ret, nil
}

func (p *sitePlugin) New(buf []byte) error {
	reader := bytes.NewReader(buf)
	r, err := zip.NewReader(reader, reader.Size())
	if err != nil {
		return err
	}
	var inst plugins.PluginInstance
	if err := json.Unmarshal([]byte(r.Comment), &inst); err != nil {
		return err
	}
	baseDir := path.Join("plugin", inst.Name)
	if utils.FileExist(baseDir) {
		return PluginExistsError
	}

	fc := func() error {
		if err := os.Mkdir(baseDir, 0755); err != nil {
			return err
		}
		for _, f := range r.File {
			if f.FileInfo().IsDir() {
				if err := os.MkdirAll(path.Join("plugin", f.Name), 0755); err != nil {
					return err
				}
			} else {
				out, err := f.Open()
				if err != nil {
					return err
				}
				defer out.Close()
				dst, err := os.OpenFile(path.Join("plugin", f.Name), os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0755)
				if err != nil {
					return err
				}
				defer dst.Close()
				if _, err := io.Copy(dst, out); err != nil {
					return err
				}
			}
		}
		return nil
	}

	if err = fc(); err != nil {
		os.RemoveAll(baseDir)
		return err
	}

	p.readPluginFolder(baseDir)
	if err := sn.Skynet.Setting.New(fmt.Sprintf("plugin_%s", inst.ID.String()), "0"); err != nil {
		log.Error(err)
	}
	if v, ok := p.plugin.Get(inst.ID); ok {
		p.checkPlugin(v)
	}
	return nil
}

func (p *sitePlugin) GetAll() interface{} {
	return &p.plugin
}

func (p *sitePlugin) Get(id uuid.UUID) interface{} {
	v, exist := p.plugin.Get(id)
	if !exist {
		return nil
	}
	return v
}

func (p *sitePlugin) Disable(id uuid.UUID) error {
	if v, exist := p.plugin.Get(id); exist {
		if !v.Enable {
			return nil
		}
		if err := v.Interface.PluginFini(); err != nil {
			return err
		}
		if err := v.Interface.PluginDisable(); err != nil {
			return err
		}
		if err := sn.Skynet.Setting.Update("plugin_"+v.ID.String(), "0"); err != nil {
			log.Error(err)
		}
		v.Enable = false
		return nil
	}
	return PluginNotFoundError
}

func (p *sitePlugin) Enable(id uuid.UUID) error {
	if v, exist := p.plugin.Get(id); exist {
		if v.Enable {
			return nil
		}
		if !p.checkPlugin(v) {
			return errors.New(v.Message)
		}
		if err := v.Interface.PluginEnable(); err != nil {
			return err
		}
		if err := v.Interface.PluginInit(); err != nil {
			return err
		}
		if err := sn.Skynet.Setting.Update("plugin_"+v.ID.String(), "1"); err != nil {
			log.Error(err)
		}
		v.Enable = true
		return nil
	}
	return PluginNotFoundError
}

func (p *sitePlugin) Fini() {
	p.plugin.Range(func(k uuid.UUID, v *PluginLoad) bool {
		if v.Enable {
			if err := v.Interface.PluginFini(); err != nil {
				log.Errorf("Plugin %v fini error: %v", v.Name, err)
			}
		}
		return true
	})
}

func (p *sitePlugin) Count() int {
	return p.plugin.Len()
}
