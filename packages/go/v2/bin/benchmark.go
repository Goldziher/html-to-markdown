// Package main provides a CLI for benchmarking the Go bindings.
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

type BenchmarkResult struct {
	Language       string  `json:"language"`
	Fixture        string  `json:"fixture"`
	FixturePath    string  `json:"fixture_path"`
	Iterations     int     `json:"iterations"`
	ElapsedSeconds float64 `json:"elapsed_seconds"`
	OpsPerSec      float64 `json:"ops_per_sec"`
	MBPerSec       float64 `json:"mb_per_sec"`
	BytesProcessed int     `json:"bytes_processed"`
}

func main() {
	filePath := flag.String("file", "", "Path to the HTML/HOCR fixture")
	iterations := flag.Int("iterations", 50, "Number of iterations")
	_ = flag.String("format", "html", "Fixture format (html or hocr)")
	flag.Parse()

	if *filePath == "" {
		fmt.Fprintln(os.Stderr, "Error: --file is required")
		os.Exit(1)
	}

	htmlBytes, err := os.ReadFile(*filePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading file: %v\n", err)
		os.Exit(1)
	}

	html := string(htmlBytes)

	// Warmup
	_, err = htmltomarkdown.Convert(html)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warmup conversion failed: %v\n", err)
		os.Exit(1)
	}

	profileOutput := os.Getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
	if profileOutput != "" {
		freq := 1000
		if env := os.Getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY"); env != "" {
			if parsed, err := strconv.Atoi(env); err == nil {
				freq = parsed
			}
		}
		if err := htmltomarkdown.StartProfiling(profileOutput, freq); err != nil {
			fmt.Fprintf(os.Stderr, "Profiling start failed: %v\n", err)
		}
	}

	// Benchmark
	start := time.Now()
	for i := 0; i < *iterations; i++ {
		_, err = htmltomarkdown.Convert(html)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Conversion failed: %v\n", err)
			os.Exit(1)
		}
	}
	elapsed := time.Since(start).Seconds()

	if profileOutput != "" {
		if err := htmltomarkdown.StopProfiling(); err != nil {
			fmt.Fprintf(os.Stderr, "Profiling stop failed: %v\n", err)
		}
	}

	bytesProcessed := len(htmlBytes) * (*iterations)
	opsPerSec := float64(*iterations) / elapsed
	mbPerSec := (float64(bytesProcessed) / (1024 * 1024)) / elapsed

	result := BenchmarkResult{
		Language:       "go",
		Fixture:        filepath.Base(*filePath),
		FixturePath:    *filePath,
		Iterations:     *iterations,
		ElapsedSeconds: elapsed,
		OpsPerSec:      opsPerSec,
		MBPerSec:       mbPerSec,
		BytesProcessed: bytesProcessed,
	}

	encoder := json.NewEncoder(os.Stdout)
	if err := encoder.Encode(result); err != nil {
		fmt.Fprintf(os.Stderr, "Error encoding JSON: %v\n", err)
		os.Exit(1)
	}
}
