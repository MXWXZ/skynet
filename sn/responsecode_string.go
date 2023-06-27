// Code generated by "stringer -type=ResponseCode"; DO NOT EDIT.

package sn

import "strconv"

func _() {
	// An "invalid array index" compiler error signifies that the constant values have changed.
	// Re-run the stringer command to generate them again.
	var x [1]struct{}
	_ = x[CodeSuccess-0]
	_ = x[CodeUserInvalid-1]
	_ = x[CodeUserNotexist-2]
	_ = x[CodeUserExist-3]
	_ = x[CodeRecaptchaInvalid-4]
	_ = x[CodeGroupNotexist-5]
	_ = x[CodeGroupExist-6]
	_ = x[CodeGroupRootupdate-7]
	_ = x[CodeGroupRootdelete-8]
	_ = x[CodePermissionNotexist-9]
	_ = x[CodePluginNotExist-10]
	_ = x[CodePluginLoaded-11]
	_ = x[CodeMax-12]
}

const _ResponseCode_name = "CodeSuccessCodeUserInvalidCodeUserNotexistCodeUserExistCodeRecaptchaInvalidCodeGroupNotexistCodeGroupExistCodeGroupRootupdateCodeGroupRootdeleteCodePermissionNotexistCodePluginNotExistCodePluginLoadedCodeMax"

var _ResponseCode_index = [...]uint8{0, 11, 26, 42, 55, 75, 92, 106, 125, 144, 166, 184, 200, 207}

func (i ResponseCode) String() string {
	if i < 0 || i >= ResponseCode(len(_ResponseCode_index)-1) {
		return "ResponseCode(" + strconv.FormatInt(int64(i), 10) + ")"
	}
	return _ResponseCode_name[_ResponseCode_index[i]:_ResponseCode_index[i+1]]
}
