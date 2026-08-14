package main

import (
	"fmt"
	"net/http"
	"os"

	azimuth "github.com/drim-dev/azimuth-go/azimuth"
)

func identity() string {
	azimuth.Realizes("polyglot/identity", "go-identifies")
	return "go"
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8081"
	}
	http.HandleFunc("/identity", func(response http.ResponseWriter, _ *http.Request) {
		fmt.Fprintln(response, identity())
	})
	if err := http.ListenAndServe(":"+port, nil); err != nil {
		panic(err)
	}
}
