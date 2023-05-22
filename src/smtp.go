package main

import (
	"bufio"
	"fmt"
	"net"
	"strings"

	"gorm.io/gorm"
)

func sendResponse(conn net.Conn, response string) {
	fmt.Fprintf(conn, "%s\r\n", response)
}

func handleConnection(conn net.Conn, db *gorm.DB) {
	defer conn.Close()

	clientAddr := conn.RemoteAddr().String()
	fmt.Printf("New connection from %s\n", clientAddr)

	sendResponse(conn, "220 haxmail")

	from := ""
	to := ""
	data := ""

	scanner := bufio.NewScanner(conn)

	for scanner.Scan() {
		request := scanner.Text()

		if request == "QUIT" {
			sendResponse(conn, "221 Bye")
			break
		}
		command := strings.Split(request, " ")[0]
		args := strings.Split(request, " ")[1:]

		switch command {
		case "HELO", "EHLO":
			sendResponse(conn, "250 Hello ")

		}
	}
}
