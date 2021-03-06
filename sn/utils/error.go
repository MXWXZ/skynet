package utils

import (
	"strings"

	log "github.com/sirupsen/logrus"
	"github.com/ztrue/tracerr"
)

// WithTrace wraps error to error with trace.
func WithTrace(err error) *log.Entry {
	return WithLogTrace(nil, err)
}

// WithLogTrace wraps error compatible to logrus
func WithLogTrace(l *log.Entry, err error) *log.Entry {
	text := tracerr.Sprint(err)
	traceText := strings.Split(text, "\n")
	if len(traceText) > 1 {
		if l != nil {
			return l.WithField("debug", traceText[1:])
		}
		return log.WithField("debug", traceText[1:])
	}
	if l != nil {
		return l.WithField("debug", nil)
	}
	return log.WithField("debug", nil)
}
