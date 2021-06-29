package api

import (
	"skynet/sn"

	"github.com/gin-gonic/gin"
	log "github.com/sirupsen/logrus"
)

func APIGetNotification(c *gin.Context, u *sn.User) (int, error) {
	var param paginationParam
	err := c.ShouldBindQuery(&param)
	if err != nil {
		return 400, err
	}

	rec, err := sn.Skynet.Notification.GetAll(&sn.SNCondition{
		Order:  []interface{}{"id " + param.Order},
		Limit:  param.Size,
		Offset: (param.Page - 1) * param.Size,
	})
	if err != nil {
		return 500, err
	}
	count, err := sn.Skynet.Notification.Count(nil)
	if err != nil {
		return 500, err
	}
	c.JSON(200, gin.H{"code": 0, "msg": "Get all notification success", "data": rec, "total": count})
	return 0, nil
}

func APIDeleteNotification(c *gin.Context, u *sn.User) (int, error) {
	fields := log.Fields{
		"ip": c.ClientIP(),
	}

	err := sn.Skynet.Notification.DeleteAll()
	if err != nil {
		return 500, err
	}
	log.WithFields(fields).Info("Delete notification")
	c.JSON(200, gin.H{"code": 0, "msg": "Delete notification success"})
	return 0, nil
}