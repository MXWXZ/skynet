package test

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPublicSetting(t *testing.T) {
	tests := []testCase{
		{
			name: "Get Public Setting",
			url:  "/setting/public",
			data: msa{
				"recaptcha.cnmirror": true,
				"recaptcha.enable":   false,
				"recaptcha.sitekey":  "test",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.test(t)
			assert.Nil(t, err)
		})
	}
}
