package main

import (
	"errors"
	plugins "skynet/plugin"
	"skynet/plugin/monitor/shared"
	"skynet/sn"
	"skynet/sn/utils"
	"sort"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	log "github.com/sirupsen/logrus"
)

// Plugin config, do NOT change the variable name
var Config = plugins.PluginConfig{
	ID:            uuid.MustParse("2eb2e1a5-66b4-45f9-ad24-3c4f05c858aa"), // go https://www.uuidgenerator.net/ to generate your plugin uuid
	Name:          "monitor",                                              // change to your plugin name
	Dependency:    []plugins.PluginDep{},                                  // if your plugin need dependency, write here
	Version:       "1.0.0",                                                // plugin version, better follow https://semver.org/
	SkynetVersion: ">= 1.0, < 1.1",                                        // skynet version constraints using https://github.com/hashicorp/go-version
	Priority:      0,                                                      // priority to run PluginInit
}

var defaultField = log.Fields{
	"plugin": Config.ID,
}

var token string

// Delete function below as you wish if you do not need
// All function will be executed after plugin loaded and dependency check

// PluginInit will be executed after plugin loaded or enabled, return error to stop skynet run or plugin enable
// Note that dependency PluginInit may NOT execute previously
func PluginInit() error {
	plugins.SPAddSubPath("Service", []*sn.SNNavItem{
		{
			Priority: 10,
			Name:     "Monitor",
			Link:     "/service/" + Config.ID.String() + "/monitor",
			Role:     sn.RoleUser,
		},
	})
	plugins.SPAddSubPath("Plugin", []*sn.SNNavItem{
		{
			Priority: 10,
			Name:     "Monitor",
			Link:     "/plugin/" + Config.ID.String(),
			Role:     sn.RoleAdmin,
		},
	})

	sn.Skynet.Page.AddPageItem([]*sn.SNPageItem{
		{
			TplName: plugins.SPWithIDPrefix(&Config, "setting"),
			Files:   plugins.SPWithLayerFiles("monitor", "setting"),
			FuncMap: sn.Skynet.Page.GetDefaultFunc(),
			Title:   "Skynet | Monitor",
			Name:    "Monitor",
			Link:    "/plugin/" + Config.ID.String(),
			Role:    sn.RoleAdmin,
			Path: sn.Skynet.Page.GetDefaultPath().WithChild([]*sn.SNPathItem{
				{
					Name: "Plugin",
					Link: "/plugin",
				},
				{
					Name:   "Monitor",
					Active: true,
				},
			}),
			Param: gin.H{
				"settingAPI": "/plugin/" + Config.ID.String(),
				"agentAPI":   "/plugin/" + Config.ID.String() + "/agent",
				"token":      token,
				"agents":     agents,
			},
		},
		{
			TplName: plugins.SPWithIDPrefix(&Config, "monitor"),
			Files:   plugins.SPWithLayerFiles("monitor", "monitor"),
			FuncMap: sn.Skynet.Page.GetDefaultFunc(),
			Title:   "Skynet | Monitor",
			Name:    "Monitor",
			Link:    "/service/" + Config.ID.String() + "/monitor",
			Role:    sn.RoleUser,
			Path: sn.Skynet.Page.GetDefaultPath().WithChild([]*sn.SNPathItem{
				{
					Name: "Service",
					Link: "#",
				},
				{
					Name:   "Monitor",
					Active: true,
				},
			}),
			Param: gin.H{
				"agentAPI": "/plugin/" + Config.ID.String() + "/agent",
			},
			BeforeReturn: func(c *gin.Context, u *sn.Users, v *sn.SNPageItem) bool {
				var sortedAgents AgentSort
				for _, v := range agents {
					sortedAgents = append(sortedAgents, v)
				}
				sort.Stable(sortedAgents)
				low, high, ok := utils.PreSplitFunc(c, v, len(sortedAgents), 10, []int{5, 10, 20, 50})
				if !ok {
					return false
				}
				if low == -1 {
					v.Param["agents"] = sortedAgents
				} else {
					v.Param["agents"] = sortedAgents[low:high]
				}
				return true
			},
		},
	})

	sn.Skynet.API.AddAPIItem([]*sn.SNAPIItem{
		{
			Path:   "/plugin/" + Config.ID.String(),
			Method: sn.APIPatch,
			Role:   sn.RoleAdmin,
			Func:   APISaveSetting,
		},
		{
			Path:   "/plugin/" + Config.ID.String() + "/agent",
			Method: sn.APIGet,
			Role:   sn.RoleUser,
			Func:   APIGetAgent,
		},
		{
			Path:   "/plugin/" + Config.ID.String() + "/agent",
			Method: sn.APIPatch,
			Role:   sn.RoleAdmin,
			Func:   APISaveAgent,
		},
		{
			Path:   "/plugin/" + Config.ID.String() + "/agent",
			Method: sn.APIDelete,
			Role:   sn.RoleAdmin,
			Func:   APIDelAgent,
		},
		{
			Path:   "/plugin/" + Config.ID.String(),
			Method: sn.APIGet,
			Role:   sn.RoleEmpty,
			Func: func(c *gin.Context, u *sn.Users) (int, error) {
				WSHandler(c.ClientIP(), c.Writer, c.Request)
				return 0, nil
			},
		},
	})

	sn.Skynet.Setting.AddSetting(plugins.SPWithIDPrefix(&Config, "token"), "")

	var exist bool
	token, exist = sn.Skynet.Setting.GetSetting(plugins.SPWithIDPrefix(&Config, "token"))
	if !exist {
		log.WithFields(defaultField).Error("Setting token not exist")
		return errors.New("Setting token not exist")
	}

	utils.GetDB().AutoMigrate(&PluginMonitorAgent{})

	var rec []PluginMonitorAgent
	err := utils.GetDB().Find(&rec).Error
	if err != nil {
		return err
	}

	for _, v := range rec {
		agents[int(v.ID)] = &shared.AgentInfo{
			ID:        int(v.ID),
			IP:        v.LastIP,
			Name:      v.Name,
			HostName:  v.Hostname,
			System:    v.System,
			Machine:   v.Machine,
			LastLogin: v.LastLogin,
			Online:    false,
		}
	}

	sn.Skynet.SharedData[plugins.SPWithIDPrefix(&Config, "")] = &pluginShared{}
	return nil
}

// PluginEnable will be executed when trigger plugin enabled
func PluginEnable() error {
	return nil
}

// PluginDisable will be executed when trigger plugin disabled, skynet will be reloaded after disabled
func PluginDisable() error {
	return nil
}
