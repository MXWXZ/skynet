package handler

import (
	"context"
	"fmt"
	"skynet/sn"
	"skynet/sn/utils"

	"github.com/ztrue/tracerr"
)

type siteUser struct{}

func NewUser() sn.SNUser {
	return &siteUser{}
}

func (u *siteUser) Count() (int64, error) {
	var count int64
	err := tracerr.Wrap(utils.GetDB().Model(&sn.User{}).Count(&count).Error)
	return count, err
}

func (u *siteUser) New(username string, password string, avatar []byte, role sn.UserRole) (string, error) {
	var newpass string
	if password == "" {
		newpass = utils.RandString(8)
	} else {
		newpass = password
	}

	webpAvatar, err := utils.ConvertWebp(avatar)
	if err != nil {
		return "", err
	}

	user := sn.User{
		Username: username,
		Password: HashPass(newpass),
		Avatar:   webpAvatar.Data,
		Role:     role,
	}
	if err := tracerr.Wrap(utils.GetDB().Create(&user).Error); err != nil {
		return "", err
	}
	err = sn.Skynet.Notification.New(sn.NotifySuccess, "User operation", "Add new user "+username+" success")
	if err != nil {
		return "", err
	}
	return newpass, nil
}

func (u *siteUser) Update(id int, username string, password string, role sn.UserRole, avatar []byte, kick bool) error {
	var err error
	if kick {
		if err := utils.DeleteSessionsByID(id); err != nil {
			return err
		}
	}
	if username == "" && password == "" && role == sn.RoleEmpty {
		return nil
	}

	var webpAvatar *utils.WebpImage
	if avatar != nil {
		webpAvatar, err = utils.ConvertWebp(avatar)
		if err != nil {
			return err
		}
	}

	var rec sn.User
	if err := tracerr.Wrap(utils.GetDB().First(&rec, id).Error); err != nil {
		return err
	}
	if username != "" {
		rec.Username = username
	}
	if password != "" {
		rec.Password = HashPass(password)
	}
	if role != sn.RoleEmpty {
		rec.Role = role
	}
	if avatar != nil {
		rec.Avatar = webpAvatar.Data
	}
	return tracerr.Wrap(utils.GetDB().Save(&rec).Error)
}

func (u *siteUser) Delete(id int) (bool, error) {
	// kick first
	if err := utils.DeleteSessionsByID(id); err != nil {
		return false, err
	}

	res := utils.GetDB().Delete(&sn.User{}, id)
	if res.RowsAffected == 0 {
		return false, nil
	} else if res.Error != nil {
		return false, tracerr.Wrap(res.Error)
	}
	err := sn.Skynet.Notification.New(sn.NotifyWarning, "User operation", fmt.Sprintf("Delete user id %v success", id))
	if err != nil {
		return false, err
	}
	return true, nil
}

func (u *siteUser) GetByUsername(username string) (*sn.User, error) {
	var rec sn.User
	err := tracerr.Wrap(utils.GetDB().Where("username = ?", username).First(&rec).Error)
	if err != nil {
		return nil, err
	}
	return &rec, nil
}

func (u *siteUser) GetByID(id int) (*sn.User, error) {
	var rec sn.User
	if err := tracerr.Wrap(utils.GetDB().First(&rec, id).Error); err != nil {
		return nil, err
	}
	return &rec, nil
}

func (u *siteUser) Reset(id int) (string, error) {
	var rec sn.User
	if err := tracerr.Wrap(utils.GetDB().First(&rec, id).Error); err != nil {
		return "", err
	}

	// ensure security, kick first
	if err := utils.DeleteSessionsByID(int(rec.ID)); err != nil {
		return "", err
	}

	newpass := utils.RandString(8)
	rec.Password = HashPass(newpass)
	if err := tracerr.Wrap(utils.GetDB().Save(&rec).Error); err != nil {
		return "", err
	}
	err := sn.Skynet.Notification.New(sn.NotifyWarning, "User operation", fmt.Sprintf("Reset user id %v success", id))
	if err != nil {
		return "", err
	}
	return newpass, nil
}

func (u *siteUser) ResetAll() (map[string]string, error) {
	var rec []sn.User
	ret := make(map[string]string)
	if err := tracerr.Wrap(utils.GetDB().Find(&rec).Error); err != nil {
		return nil, err
	}
	if len(rec) == 0 {
		return ret, nil
	}

	// ensure security, kick first
	if err := tracerr.Wrap(utils.GetRedis().FlushDB(context.Background()).Err()); err != nil {
		return nil, err
	}

	for i := range rec {
		newpass := utils.RandString(8)
		rec[i].Password = HashPass(newpass)
		ret[rec[i].Username] = newpass
	}

	if err := tracerr.Wrap(utils.GetDB().Save(&rec).Error); err != nil {
		return nil, err
	}
	return ret, nil
}

func (u *siteUser) GetAll(cond *sn.SNCondition) ([]*sn.User, error) {
	var ret []*sn.User
	return ret, tracerr.Wrap(utils.DBParseCondition(cond).Find(&ret).Error)
}
