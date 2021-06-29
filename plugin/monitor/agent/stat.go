package main

import (
	"context"
	"encoding/json"
	"skynet/plugin/monitor/msg"
	"time"

	"github.com/gorilla/websocket"
	"github.com/shirou/gopsutil/cpu"
	"github.com/shirou/gopsutil/disk"
	"github.com/shirou/gopsutil/load"
	"github.com/shirou/gopsutil/mem"
	"github.com/shirou/gopsutil/net"
	log "github.com/sirupsen/logrus"
)

func UploadStat(ctx context.Context, c *websocket.Conn) {
	ticker := time.NewTicker(1 * time.Second)
	for {
		select {
		case <-ticker.C:
			cpuUsage, err := cpu.Percent(0, false)
			if err != nil {
				log.Warn("Could not determine cpu usage")
			}
			memUsage, err := mem.VirtualMemory()
			if err != nil {
				log.Warn("Could not determine mem usage")
			}
			partionUsage, err := disk.Partitions(false)
			if err != nil {
				log.Warn("Could not determine disk usage")
			}
			var diskUsage, disktotUsage uint64
			for _, v := range partionUsage {
				usage, err := disk.Usage(v.Mountpoint)
				if err != nil {
					log.Warn("Could not determine disk usage")
				}
				diskUsage += usage.Used
				disktotUsage += usage.Total
			}
			loadUsage, err := load.Avg()
			if err != nil {
				log.Warn("Could not determine load usage")
			}
			netUsage, err := net.IOCounters(false)
			if err != nil {
				log.Warn("Could not determine net usage")
			}

			d, err := json.Marshal(msg.StatMsg{
				CPU:       cpuUsage[0],
				Mem:       memUsage.Used,
				TotalMem:  memUsage.Total,
				Disk:      diskUsage,
				TotalDisk: disktotUsage,
				Load1:     loadUsage.Load1,
				Time:      time.Now(),
				BandUp:    netUsage[0].BytesSent,
				BandDown:  netUsage[0].BytesRecv,
			})
			if err != nil {
				log.Fatal(err)
			}
			_, err = msg.SendReq(c, msg.OPStat, string(d))
			if err != nil {
				log.Warn(err)
			}
		case <-ctx.Done():
			ticker.Stop()
			return
		}
	}
}
