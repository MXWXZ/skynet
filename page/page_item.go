package page

import (
	"fmt"
	"path"
	"skynet/sn"
	"skynet/sn/utils"

	"github.com/gin-gonic/gin"
	"github.com/spf13/viper"
)

var defaultPath = &sn.SNPathItem{
	Name: "Home",
	Link: "/",
}

func withLayerFiles(n string) []string {
	return []string{"templates/home.tmpl", path.Join("templates", n), "templates/header.tmpl", "templates/footer.tmpl"}
}

func withSingleFiles(n string) []string {
	return []string{path.Join("templates", n), "templates/header.tmpl", "templates/footer.tmpl"}
}

var pages = []*sn.SNPageItem{
	{
		TplName: "dashboard",
		Files:   withLayerFiles("dashboard.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet | Dashboard",
		Name:    "Dashboard",
		Link:    "/dashboard",
		Role:    sn.RoleUser,
		Path: defaultPath.WithChild([]*sn.SNPathItem{
			{
				Name:   "Dashboard",
				Active: true,
			},
		}),
	},
	{
		TplName: "index",
		Files:   withSingleFiles("index.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet",
		Link:    "/",
		Role:    sn.RoleEmpty,
		BeforeRender: func(c *gin.Context, u *sn.User, v *sn.SNPageItem) bool {
			v.Param["reSwitch"] = viper.GetBool("recaptcha.enable")
			v.Param["reMirror"] = viper.GetBool("recaptcha.cnmirror")
			v.Param["reSitekey"] = viper.GetString("recaptcha.sitekey")
			if data, err := c.Cookie(viper.GetString("session.cookie")); err == nil && data != "" {
				c.Redirect(302, "/dashboard")
				return false
			}
			return true
		},
	},
	{
		TplName: "plugin",
		Files:   withLayerFiles("plugin.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet | Plugin Manager",
		Name:    "Plugin Manager",
		Link:    "/plugin",
		Role:    sn.RoleAdmin,
		Path: defaultPath.WithChild([]*sn.SNPathItem{
			{
				Name: "Plugin",
				Link: "/plugin",
			},
			{
				Name:   "Manager",
				Active: true,
			},
		}),
		BeforeRender: func(c *gin.Context, u *sn.User, v *sn.SNPageItem) bool {
			v.Param["_total"] = sn.Skynet.Plugin.Count()
			return true
		},
	},
	{
		TplName: "setting",
		Files:   withLayerFiles("setting.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet | Setting",
		Name:    "Setting",
		Link:    "/setting",
		Role:    sn.RoleUser,
		Path: defaultPath.WithChild([]*sn.SNPathItem{
			{
				Name:   "Setting",
				Active: true,
			},
		}),
	},
	{
		TplName: "deny",
		Files:   withLayerFiles("deny.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet | Permission Denied",
		Name:    "Permission Denied",
		Link:    "/deny",
		Role:    sn.RoleUser,
		Path: defaultPath.WithChild([]*sn.SNPathItem{
			{
				Name:   "Permission Denied",
				Active: true,
			},
		}),
	},
	{
		TplName: "user",
		Files:   withLayerFiles("user.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet | User",
		Name:    "User",
		Link:    "/user",
		Role:    sn.RoleAdmin,
		Path: defaultPath.WithChild([]*sn.SNPathItem{
			{
				Name:   "User",
				Active: true,
			},
		}),
		BeforeRender: func(c *gin.Context, u *sn.User, v *sn.SNPageItem) bool {
			count, err := sn.Skynet.User.Count()
			if err != nil {
				utils.WithTrace(err).Error(err)
				c.AbortWithStatus(500)
				return false
			}
			v.Param["_total"] = count
			return true
		},
	},
	{
		TplName: "notification",
		Files:   withLayerFiles("notification.tmpl"),
		FuncMap: defaultFunc,
		Title:   "Skynet | Notification",
		Name:    "Notification",
		Link:    "/notification",
		Role:    sn.RoleUser,
		Path: defaultPath.WithChild([]*sn.SNPathItem{
			{
				Name:   "Notification",
				Active: true,
			},
		}),
		BeforeRender: func(c *gin.Context, u *sn.User, v *sn.SNPageItem) bool {
			count, err := sn.Skynet.Notification.Count(nil)
			if err != nil {
				utils.WithTrace(err).Error(err)
				c.AbortWithStatus(500)
				return false
			}
			v.Param["_total"] = count
			err = sn.Skynet.Notification.MarkAllRead()
			if err != nil {
				utils.WithTrace(err).Error(err)
				c.AbortWithStatus(500)
				return false
			}
			return true
		},
	},
}

var navbar = []*sn.SNNavItem{
	{
		ID:   "skynet-dashboard",
		Name: "Dashboard",
		Link: "/dashboard",
		Icon: "fa-tachometer-alt",
		Role: sn.RoleUser,
	},
	{
		ID:   "skynet-service",
		Name: "Service",
		Link: "#",
		Icon: "fa-briefcase",
		Role: sn.RoleUser,
	},
	{
		ID:   "skynet-plugin",
		Name: "Plugin",
		Link: "#",
		Icon: "fa-plug",
		Role: sn.RoleAdmin,
		Child: []*sn.SNNavItem{
			{
				ID:   "skynet-manager",
				Name: "Manager",
				Link: "/plugin",
				Role: sn.RoleAdmin,
			},
		},
	},
	{
		ID:         "skynet-notification",
		Name:       "Notification",
		Link:       "/notification",
		Icon:       "fa-bell",
		Role:       sn.RoleUser,
		BadgeClass: "badge-warning",
		RenderPrepare: func(c *gin.Context, s *sn.SNNavItem, l []*sn.SNNavItem) bool {
			count, err := sn.Skynet.Notification.Count(false)
			if err != nil {
				utils.WithTrace(err).Error(err)
				c.AbortWithStatus(500)
				return false
			}
			c.Set("_notification_unread", count)
			if count != 0 && !s.Active {
				s.Badge = fmt.Sprint(count)
			}
			return true
		},
	},
	{
		ID:   "skynet-user",
		Name: "User",
		Link: "/user",
		Icon: "fa-user",
		Role: sn.RoleAdmin,
	},
	{
		ID:   "skynet-setting",
		Name: "Setting",
		Link: "/setting",
		Icon: "fa-cog",
		Role: sn.RoleUser,
	},
}
