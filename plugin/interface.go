package plugins

import (
	"skynet/sn"

	"github.com/google/uuid"
)

type PluginDep struct {
	ID      uuid.UUID
	Name    string
	Version string
}

type PluginConfig struct {
	ID            uuid.UUID
	Name          string
	Dependency    []PluginDep
	Path          string
	Version       string
	SkynetVersion string
}

func SPWithIDPrefix(c *PluginConfig, n string) string {
	if n != "" {
		return "plugin_" + c.ID.String() + "_" + n
	} else {
		return "plugin_" + c.ID.String()
	}
}

func SPAddSubPath(root string, i []*sn.SNNavItem) {
	for _, v := range sn.Skynet.Page.GetNavItem() {
		if v.Name == root {
			v.Child = append(v.Child, i...)
			sn.SNNavSort(v.Child).Sort()
		}
	}
}

func SPWithLayerFiles(pn string, n string) []string {
	return []string{"templates/home.tmpl", "plugin/" + pn + "/templates/" + n + ".tmpl", "templates/header.tmpl", "templates/footer.tmpl"}
}

func SPWithSingleFiles(pn string, n string) []string {
	return []string{"plugin/" + pn + "/templates/" + n + ".tmpl", "templates/header.tmpl", "templates/footer.tmpl"}
}
