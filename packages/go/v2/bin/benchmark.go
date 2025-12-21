// Package main provides a CLI for benchmarking the Go bindings.
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

type BenchmarkResult struct {
	Language       string  `json:"language"`
	Fixture        string  `json:"fixture"`
	FixturePath    string  `json:"fixture_path"`
	Scenario       string  `json:"scenario"`
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
	scenario := flag.String("scenario", "convert-default", "Scenario to benchmark")
	flag.Parse()

	if *filePath == "" {
		fmt.Fprintln(os.Stderr, "Error: --file is required")
		os.Exit(1)
	}

	if *scenario != "convert-default" && *scenario != "metadata-default" {
		fmt.Fprintf(os.Stderr, "Unsupported scenario: %s\n", *scenario)
		os.Exit(1)
	}

	htmlBytes, err := os.ReadFile(*filePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading file: %v\n", err)
		os.Exit(1)
	}

	html := string(htmlBytes)

	profileOutput := os.Getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
	profileFrequency := os.Getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY")
	profileOnce := os.Getenv("HTML_TO_MARKDOWN_PROFILE_ONCE")
	profileRepeat := os.Getenv("HTML_TO_MARKDOWN_PROFILE_REPEAT")
	if profileOutput != "" {
		_ = os.Unsetenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
		_ = os.Unsetenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY")
		_ = os.Unsetenv("HTML_TO_MARKDOWN_PROFILE_ONCE")
		_ = os.Unsetenv("HTML_TO_MARKDOWN_PROFILE_REPEAT")
	}

	// Warmup (avoid profiling the warmup run).
	err = runScenario(html, *scenario)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warmup conversion failed: %v\n", err)
		os.Exit(1)
	}

	if profileOutput != "" {
		_ = os.Setenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT", profileOutput)
		if profileFrequency != "" {
			_ = os.Setenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY", profileFrequency)
		}
		if profileOnce != "" {
			_ = os.Setenv("HTML_TO_MARKDOWN_PROFILE_ONCE", profileOnce)
		}
		if profileRepeat != "" {
			_ = os.Setenv("HTML_TO_MARKDOWN_PROFILE_REPEAT", profileRepeat)
		}
	}

	// Benchmark
	start := time.Now()
	for i := 0; i < *iterations; i++ {
		err = runScenario(html, *scenario)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Conversion failed: %v\n", err)
			os.Exit(1)
		}
	}
	elapsed := time.Since(start).Seconds()

	bytesProcessed := len(htmlBytes) * (*iterations)
	opsPerSec := float64(*iterations) / elapsed
	mbPerSec := (float64(bytesProcessed) / (1024 * 1024)) / elapsed

	result := BenchmarkResult{
		Language:       "go",
		Fixture:        filepath.Base(*filePath),
		FixturePath:    *filePath,
		Scenario:       *scenario,
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

func runScenario(html string, scenario string) error {
	switch scenario {
	case "metadata-default":
		_, err := htmltomarkdown.ConvertWithMetadata(html)
		return err
	default:
		_, err := htmltomarkdown.Convert(html)
		return err
	}
}
