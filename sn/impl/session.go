package impl

import (
	"context"
	"encoding/gob"
	"fmt"
	"skynet/sn"
	"skynet/sn/tpl"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"github.com/google/uuid"
	"github.com/gorilla/sessions"
	"github.com/spf13/viper"
	"github.com/ztrue/tracerr"
)

func init() {
	gob.Register(uuid.UUID{})
}

var ErrSessionInvalid = tracerr.New("session invalid")

type SessionData struct {
	ID uuid.UUID `gob:"id"`
}

func (s *SessionData) SaveSession(session *sessions.Session) {
	session.Values["id"] = s.ID
}

func LoadSession(session *sessions.Session) (*SessionData, error) {
	var tmp SessionData
	var ok bool
	tmp.ID, ok = session.Values["id"].(uuid.UUID)
	if !ok {
		return nil, fmt.Errorf("%w: id not found", ErrSessionInvalid)
	}
	return &tmp, nil
}

func DecodeSessionString(s string) (*SessionData, error) {
	var tmp map[any]any
	dec := gob.NewDecoder(strings.NewReader(s))
	if err := tracerr.Wrap(dec.Decode(&tmp)); err != nil {
		return nil, err
	}
	return &SessionData{
		ID: tmp["id"].(uuid.UUID),
	}, nil
}

// GetCTXSession gets session object from gin context.
func GetCTXSession(c *gin.Context) (*sessions.Session, error) {
	ret, err := sn.Skynet.GetSession().Get(c.Request, viper.GetString("session.cookie"))
	return ret, tracerr.Wrap(err)
}

// SaveCTXSession saves session object to gin context.
func SaveCTXSession(c *gin.Context) error {
	return tracerr.Wrap(sessions.Save(c.Request, c.Writer))
}

// FindSessionsByID find all sessions associate to user by uid. When uid = nil,
// return all sessions.
func FindSessions(uid []uuid.UUID) (map[string]*SessionData, error) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Duration(viper.GetInt("session.timeout"))*time.Second)
	defer cancel()

	ret := make(map[string]*SessionData)
	res, err := sn.Skynet.GetRedis().Keys(ctx, fmt.Sprintf("%v*", viper.GetString("session.prefix"))).Result()
	if err != nil {
		return nil, tracerr.Wrap(err)
	}
	if len(res) == 0 {
		return ret, nil
	}

	// pipeline to accelerate
	pipe := sn.Skynet.GetRedis().Pipeline()
	piperes := make(map[string]*redis.StringCmd)
	for _, v := range res {
		piperes[v] = pipe.Get(ctx, v)
	}
	_, err = pipe.Exec(ctx)
	if err != nil {
		return nil, tracerr.Wrap(err)
	}
	for _, v := range piperes {
		if v.Err() != nil {
			return nil, tracerr.Wrap(v.Err())
		}
	}

	// since redis store gob serialized data, we need to decode first
	look := tpl.NewSliceFinder(uid)
	for k, v := range piperes {
		tmp, err := DecodeSessionString(v.Val())
		if err != nil {
			return nil, err
		}
		if uid == nil || look.Find(tmp.ID) {
			ret[k] = tmp
		}
	}
	return ret, nil
}

// DeleteSessions deletes all sessions by uid. This equals kick user operation.
//
// If uid==uuid.Nil, delete all sessions.
func DeleteSessions(uid []uuid.UUID) error {
	ctx, cancel := context.WithTimeout(context.Background(), time.Duration(viper.GetInt("session.timeout"))*time.Second)
	defer cancel()
	if uid == nil {
		return tracerr.Wrap(sn.Skynet.GetRedis().FlushDB(ctx).Err())
	}

	s, err := FindSessions(uid)
	if err != nil {
		return err
	}

	pipe := sn.Skynet.GetRedis().Pipeline()
	piperes := make(map[string]*redis.IntCmd)
	for k := range s {
		piperes[k] = pipe.Del(ctx, k)
	}
	_, err = pipe.Exec(ctx)
	if err != nil {
		return tracerr.Wrap(err)
	}
	for _, v := range piperes {
		if v.Err() != nil {
			return tracerr.Wrap(v.Err())
		}
	}
	return nil
}

func GetSessionData(c *gin.Context) (*SessionData, error) {
	data, ok := c.Get("session")
	if !ok {
		return nil, ErrSessionInvalid
	}
	return data.(*SessionData), nil
}
