package services

import (
	"bufio"
	"fmt"
	"net"
	"strings"
)

const (
	StateInit string = "init"
	StateHelo        = "helo"
	StateMail        = "mailto"
	StateRcpt        = "rcptto"
	StateData        = "data"
	StateQuit        = "quit"
)

func sendResponse(conn net.Conn, response string) {
	fmt.Fprintf(conn, "%s\r\n", response)
}

func HandleClient(conn net.Conn) {
	defer conn.Close()

	state := StateInit
	sendResponse(conn, "220 Haxmail")
	state = StateHelo
	from := ""
	to := ""
	data := ""
	scanner := bufio.NewScanner(conn)

	for scanner.Scan() {
		line := scanner.Text()
		//TODO: handle case for each condition
		line = strings.ToUpper(line)
		lines := strings.Fields(line)
		fmt.Println(len(lines), "    ", lines)
		switch state {
		case StateHelo:
			if strings.HasPrefix(line, "HELO") || strings.HasPrefix(line, "EHLO") {
				fmt.Println("HELO received")
				sendResponse(conn, "250 Haxmail")
				state = StateMail
			} else {
				fmt.Println(line)
				sendResponse(conn, "500 invalid command")
			}
		case StateMail:
			if strings.HasPrefix(line, "MAIL FROM:") {
				from = line
				sendResponse(conn, "250 OK")
				state = StateRcpt
			} else {
				sendResponse(conn, "500 invalid command")
			}
		case StateRcpt:
			if strings.HasPrefix(line, "RCPT TO:") {
				to = line
				sendResponse(conn, "250 OK")
			} else if strings.HasPrefix(line, "DATA") {
				sendResponse(conn, "354 Start mail input; end with <CRLF>.<CRLF>")
				state = StateData
			} else {
				sendResponse(conn, "500 invalid command")
			}
		case StateData:
			if line == "." {
				sendResponse(conn, "250 OK")
				fmt.Printf("From: %s\nTo: %s\nData:\n%s\n", from, to, data)
				state = StateQuit
			} else {
				data += line + "\n"
			}
		case StateQuit:
			sendResponse(conn, "221 Bye")
			return
		}
	}

	if err := scanner.Err(); err != nil {
		fmt.Printf("Error reading from connection: %s\n", err)
	}
}
