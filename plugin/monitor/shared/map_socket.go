// This file was automatically generated by genny.
// Any changes will be lost if this file is regenerated.
// see https://github.com/cheekybits/genny

package shared

import (
	"sort"
	"sync"

	"github.com/google/uuid"
)

type SocketElement struct {
	Key   uuid.UUID
	Value *Websocket
}

type SocketSorterFunc func(a *SocketElement, b *SocketElement) bool

type SocketMap struct {
	Data   sync.Map
	length int
}

func (m *SocketMap) Clear() {
	m.length = 0
	m.Data = sync.Map{}
}

func (m *SocketMap) Len() int {
	return m.length
}

func (m *SocketMap) Get(k uuid.UUID) (*Websocket, bool) {
	ret, ok := m.Data.Load(k)
	if ret == nil {
		return nil, false
	}
	return ret.(*Websocket), ok
}

func (m *SocketMap) MustGet(k uuid.UUID) *Websocket {
	ret, ok := m.Get(k)
	if !ok {
		panic("key not found")
	}
	return ret
}

func (m *SocketMap) Set(k uuid.UUID, v *Websocket) {
	if !m.Has(k) {
		m.length++
	}
	m.Data.Store(k, v)
}

func (m *SocketMap) SetIfAbsent(k uuid.UUID, v *Websocket) (*Websocket, bool) {
	ret, ok := m.Data.LoadOrStore(k, v)
	if !ok {
		m.length++
	}
	if ret == nil {
		panic("key not found")
	}
	return ret.(*Websocket), ok
}

func (m *SocketMap) Delete(k uuid.UUID) {
	if m.Has(k) {
		m.length--
	}
	m.Data.Delete(k)
}

func (m *SocketMap) Has(k uuid.UUID) bool {
	_, ok := m.Data.Load(k)
	return ok
}

func (m *SocketMap) Range(f func(k uuid.UUID, v *Websocket) bool) {
	m.Data.Range(func(key, value interface{}) bool {
		return f(key.(uuid.UUID), value.(*Websocket))
	})
}

func (m *SocketMap) Keys() []uuid.UUID {
	var ret []uuid.UUID
	m.Range(func(key uuid.UUID, value *Websocket) bool {
		ret = append(ret, key)
		return true
	})
	return ret
}

func (m *SocketMap) Values() []*Websocket {
	var ret []*Websocket
	m.Range(func(key uuid.UUID, value *Websocket) bool {
		ret = append(ret, value)
		return true
	})
	return ret
}

func (m *SocketMap) Elements() []*SocketElement {
	var ret []*SocketElement
	m.Range(func(key uuid.UUID, value *Websocket) bool {
		ret = append(ret, &SocketElement{
			Key:   key,
			Value: value,
		})
		return true
	})
	return ret
}

func (m *SocketMap) Map() map[uuid.UUID]*Websocket {
	ret := make(map[uuid.UUID]*Websocket)
	m.Range(func(key uuid.UUID, value *Websocket) bool {
		ret[key] = value
		return true
	})
	return ret
}

func (m *SocketMap) SortElement(f SocketSorterFunc) []*SocketElement {
	ret := m.Elements()
	sort.SliceStable(ret, func(i, j int) bool {
		return f(ret[i], ret[j])
	})
	return ret
}

func (m *SocketMap) SortKey(f SocketSorterFunc) []uuid.UUID {
	res := m.SortElement(f)
	var ret []uuid.UUID
	for _, v := range res {
		ret = append(ret, v.Key)
	}
	return ret
}

func (m *SocketMap) SortValue(f SocketSorterFunc) []*Websocket {
	res := m.SortElement(f)
	var ret []*Websocket
	for _, v := range res {
		ret = append(ret, v.Value)
	}
	return ret
}
