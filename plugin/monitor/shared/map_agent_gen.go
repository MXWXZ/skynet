// This file was automatically generated by genny.
// Any changes will be lost if this file is regenerated.
// see https://github.com/cheekybits/genny

package shared

import (
	"sort"
	"sync"
)

type AgentElement struct {
	Key   int
	Value *AgentInfo
}

type AgentSorterFunc func(a *AgentElement, b *AgentElement) bool

type AgentMap struct {
	Data sync.Map
}

func (m *AgentMap) Clear() {
	m.Data = sync.Map{}
}

func (m *AgentMap) Len() int {
	return len(m.Keys())
}

func (m *AgentMap) Get(k int) (*AgentInfo, bool) {
	ret, ok := m.Data.Load(k)
	if ret == nil {
		return nil, false
	}
	return ret.(*AgentInfo), ok
}

func (m *AgentMap) MustGet(k int) *AgentInfo {
	ret, ok := m.Get(k)
	if !ok {
		panic("key not found")
	}
	return ret
}

func (m *AgentMap) Set(k int, v *AgentInfo) {
	m.Data.Store(k, v)
}

func (m *AgentMap) SetIfAbsent(k int, v *AgentInfo) (*AgentInfo, bool) {
	ret, ok := m.Data.LoadOrStore(k, v)
	if ret == nil {
		panic("value is nil")
	}
	return ret.(*AgentInfo), ok
}

func (m *AgentMap) Delete(k int) {
	m.Data.Delete(k)
}

func (m *AgentMap) Has(k int) bool {
	_, ok := m.Data.Load(k)
	return ok
}

func (m *AgentMap) Range(f func(k int, v *AgentInfo) bool) {
	m.Data.Range(func(key, value interface{}) bool {
		return f(key.(int), value.(*AgentInfo))
	})
}

func (m *AgentMap) Keys() []int {
	var ret []int
	m.Range(func(key int, value *AgentInfo) bool {
		ret = append(ret, key)
		return true
	})
	return ret
}

func (m *AgentMap) Values() []*AgentInfo {
	var ret []*AgentInfo
	m.Range(func(key int, value *AgentInfo) bool {
		ret = append(ret, value)
		return true
	})
	return ret
}

func (m *AgentMap) Elements() []*AgentElement {
	var ret []*AgentElement
	m.Range(func(key int, value *AgentInfo) bool {
		ret = append(ret, &AgentElement{
			Key:   key,
			Value: value,
		})
		return true
	})
	return ret
}

func (m *AgentMap) Map() map[int]*AgentInfo {
	ret := make(map[int]*AgentInfo)
	m.Range(func(key int, value *AgentInfo) bool {
		ret[key] = value
		return true
	})
	return ret
}

func (m *AgentMap) SortElement(f AgentSorterFunc) []*AgentElement {
	ret := m.Elements()
	sort.SliceStable(ret, func(i, j int) bool {
		return f(ret[i], ret[j])
	})
	return ret
}

func (m *AgentMap) SortKey(f AgentSorterFunc) []int {
	res := m.SortElement(f)
	var ret []int
	for _, v := range res {
		ret = append(ret, v.Key)
	}
	return ret
}

func (m *AgentMap) SortValue(f AgentSorterFunc) []*AgentInfo {
	res := m.SortElement(f)
	var ret []*AgentInfo
	for _, v := range res {
		ret = append(ret, v.Value)
	}
	return ret
}