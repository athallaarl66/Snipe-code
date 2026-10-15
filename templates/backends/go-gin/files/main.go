package main

import (
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
)

// NewRouter builds the HTTP router, so tests can drive it without a listener.
func NewRouter() *gin.Engine {
	r := gin.Default()
	r.GET("/", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "Hello World"})
	})

	return r
}

func main() {
	if err := NewRouter().Run(":8080"); err != nil {
		log.Fatal(err)
	}
}
