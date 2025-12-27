package htmltomarkdown

// #cgo linux LDFLAGS: -ldl
// #cgo windows LDFLAGS: -lkernel32
// #include <stdlib.h>
// #include <stdbool.h>
// #include <stdint.h>
// #if defined(_WIN32)
// #include <windows.h>
// static HMODULE ffi_handle = NULL;
// static FARPROC html_to_markdown_convert_ptr = NULL;
// static FARPROC html_to_markdown_free_string_ptr = NULL;
// static FARPROC html_to_markdown_version_ptr = NULL;
// static FARPROC html_to_markdown_last_error_ptr = NULL;
// static FARPROC html_to_markdown_convert_with_metadata_ptr = NULL;
// static FARPROC html_to_markdown_profile_start_ptr = NULL;
// static FARPROC html_to_markdown_profile_stop_ptr = NULL;
// static FARPROC html_to_markdown_convert_with_visitor_ptr = NULL;
// static FARPROC html_to_markdown_visitor_create_ptr = NULL;
// static FARPROC html_to_markdown_visitor_free_ptr = NULL;
//
// bool html_to_markdown_ffi_load(const char* path) {
// 	ffi_handle = LoadLibraryA(path);
// 	if (!ffi_handle) {
// 		return false;
// 	}
// 	html_to_markdown_convert_ptr = GetProcAddress(ffi_handle, "html_to_markdown_convert");
// 	html_to_markdown_free_string_ptr = GetProcAddress(ffi_handle, "html_to_markdown_free_string");
// 	html_to_markdown_version_ptr = GetProcAddress(ffi_handle, "html_to_markdown_version");
// 	html_to_markdown_last_error_ptr = GetProcAddress(ffi_handle, "html_to_markdown_last_error");
// 	html_to_markdown_convert_with_metadata_ptr = GetProcAddress(ffi_handle, "html_to_markdown_convert_with_metadata");
// 	html_to_markdown_profile_start_ptr = GetProcAddress(ffi_handle, "html_to_markdown_profile_start");
// 	html_to_markdown_profile_stop_ptr = GetProcAddress(ffi_handle, "html_to_markdown_profile_stop");
// 	html_to_markdown_convert_with_visitor_ptr = GetProcAddress(ffi_handle, "html_to_markdown_convert_with_visitor");
// 	html_to_markdown_visitor_create_ptr = GetProcAddress(ffi_handle, "html_to_markdown_visitor_create");
// 	html_to_markdown_visitor_free_ptr = GetProcAddress(ffi_handle, "html_to_markdown_visitor_free");
// 	if (!html_to_markdown_convert_ptr || !html_to_markdown_free_string_ptr ||
// 		!html_to_markdown_version_ptr || !html_to_markdown_last_error_ptr ||
// 		!html_to_markdown_convert_with_metadata_ptr || !html_to_markdown_profile_start_ptr ||
// 		!html_to_markdown_profile_stop_ptr || !html_to_markdown_convert_with_visitor_ptr ||
// 		!html_to_markdown_visitor_create_ptr || !html_to_markdown_visitor_free_ptr) {
// 		FreeLibrary(ffi_handle);
// 		ffi_handle = NULL;
// 		return false;
// 	}
// 	return true;
// }
// #else
// #include <dlfcn.h>
// static void* ffi_handle = NULL;
// static void* html_to_markdown_convert_ptr = NULL;
// static void* html_to_markdown_free_string_ptr = NULL;
// static void* html_to_markdown_version_ptr = NULL;
// static void* html_to_markdown_last_error_ptr = NULL;
// static void* html_to_markdown_convert_with_metadata_ptr = NULL;
// static void* html_to_markdown_profile_start_ptr = NULL;
// static void* html_to_markdown_profile_stop_ptr = NULL;
// static void* html_to_markdown_convert_with_visitor_ptr = NULL;
// static void* html_to_markdown_visitor_create_ptr = NULL;
// static void* html_to_markdown_visitor_free_ptr = NULL;
//
// bool html_to_markdown_ffi_load(const char* path) {
// 	ffi_handle = dlopen(path, RTLD_LAZY);
// 	if (!ffi_handle) {
// 		return false;
// 	}
// 	html_to_markdown_convert_ptr = dlsym(ffi_handle, "html_to_markdown_convert");
// 	html_to_markdown_free_string_ptr = dlsym(ffi_handle, "html_to_markdown_free_string");
// 	html_to_markdown_version_ptr = dlsym(ffi_handle, "html_to_markdown_version");
// 	html_to_markdown_last_error_ptr = dlsym(ffi_handle, "html_to_markdown_last_error");
// 	html_to_markdown_convert_with_metadata_ptr = dlsym(ffi_handle, "html_to_markdown_convert_with_metadata");
// 	html_to_markdown_profile_start_ptr = dlsym(ffi_handle, "html_to_markdown_profile_start");
// 	html_to_markdown_profile_stop_ptr = dlsym(ffi_handle, "html_to_markdown_profile_stop");
// 	html_to_markdown_convert_with_visitor_ptr = dlsym(ffi_handle, "html_to_markdown_convert_with_visitor");
// 	html_to_markdown_visitor_create_ptr = dlsym(ffi_handle, "html_to_markdown_visitor_create");
// 	html_to_markdown_visitor_free_ptr = dlsym(ffi_handle, "html_to_markdown_visitor_free");
// 	if (!html_to_markdown_convert_ptr || !html_to_markdown_free_string_ptr ||
// 		!html_to_markdown_version_ptr || !html_to_markdown_last_error_ptr ||
// 		!html_to_markdown_convert_with_metadata_ptr || !html_to_markdown_profile_start_ptr ||
// 		!html_to_markdown_profile_stop_ptr || !html_to_markdown_convert_with_visitor_ptr ||
// 		!html_to_markdown_visitor_create_ptr || !html_to_markdown_visitor_free_ptr) {
// 		dlclose(ffi_handle);
// 		ffi_handle = NULL;
// 		return false;
// 	}
// 	return true;
// }
// #endif
//
// static const char* html_to_markdown_ffi_error = "html-to-markdown FFI library not loaded";
//
// typedef char* (*convert_fn)(const char*);
// typedef void (*free_string_fn)(char*);
// typedef const char* (*version_fn)(void);
// typedef const char* (*last_error_fn)(void);
// typedef char* (*convert_with_metadata_fn)(const char*, char**);
// typedef bool (*profile_start_fn)(const char*, int32_t);
// typedef bool (*profile_stop_fn)(void);
// typedef char* (*convert_with_visitor_fn)(const char*, void*);
// typedef void* (*visitor_create_fn)(const void*);
// typedef void (*visitor_free_fn)(void*);
//
// char* html_to_markdown_convert_proxy(const char* html) {
// 	if (!html_to_markdown_convert_ptr) {
// 		return NULL;
// 	}
// 	return ((convert_fn)html_to_markdown_convert_ptr)(html);
// }
//
// void html_to_markdown_free_string_proxy(char* s) {
// 	if (!html_to_markdown_free_string_ptr) {
// 		return;
// 	}
// 	((free_string_fn)html_to_markdown_free_string_ptr)(s);
// }
//
// const char* html_to_markdown_version_proxy(void) {
// 	if (!html_to_markdown_version_ptr) {
// 		return NULL;
// 	}
// 	return ((version_fn)html_to_markdown_version_ptr)();
// }
//
// const char* html_to_markdown_last_error_proxy(void) {
// 	if (!html_to_markdown_last_error_ptr) {
// 		return html_to_markdown_ffi_error;
// 	}
// 	return ((last_error_fn)html_to_markdown_last_error_ptr)();
// }
//
// char* html_to_markdown_convert_with_metadata_proxy(const char* html, char** metadata_json) {
// 	if (!html_to_markdown_convert_with_metadata_ptr) {
// 		return NULL;
// 	}
// 	return ((convert_with_metadata_fn)html_to_markdown_convert_with_metadata_ptr)(html, metadata_json);
// }
//
// bool html_to_markdown_profile_start_proxy(const char* output, int32_t frequency) {
// 	if (!html_to_markdown_profile_start_ptr) {
// 		return false;
// 	}
// 	return ((profile_start_fn)html_to_markdown_profile_start_ptr)(output, frequency);
// }
//
// bool html_to_markdown_profile_stop_proxy(void) {
// 	if (!html_to_markdown_profile_stop_ptr) {
// 		return false;
// 	}
// 	return ((profile_stop_fn)html_to_markdown_profile_stop_ptr)();
// }
//
// char* html_to_markdown_convert_with_visitor_proxy(const char* html, void* visitor) {
// 	if (!html_to_markdown_convert_with_visitor_ptr) {
// 		return NULL;
// 	}
// 	return ((convert_with_visitor_fn)html_to_markdown_convert_with_visitor_ptr)(html, visitor);
// }
//
// void* html_to_markdown_visitor_create_proxy(const void* callbacks) {
// 	if (!html_to_markdown_visitor_create_ptr) {
// 		return NULL;
// 	}
// 	return ((visitor_create_fn)html_to_markdown_visitor_create_ptr)(callbacks);
// }
//
// void html_to_markdown_visitor_free_proxy(void* visitor) {
// 	if (!html_to_markdown_visitor_free_ptr) {
// 		return;
// 	}
// 	((visitor_free_fn)html_to_markdown_visitor_free_ptr)(visitor);
// }
import "C"

import (
	"archive/tar"
	"archive/zip"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"
	"unsafe"
)

const (
	defaultFFIVersion = "2.16.0"
	githubRepo        = "Goldziher/html-to-markdown"

	archAMD64    = "amd64"
	archARM64    = "arm64"
	archiveTarGz = "tar.gz"
)

var (
	ffiLoadOnce sync.Once
	ffiLoadErr  error
)

func ensureFFILoaded() error {
	ffiLoadOnce.Do(func() {
		ffiLoadErr = loadFFI()
	})
	return ffiLoadErr
}

func loadFFI() error {
	if path := os.Getenv("HTML_TO_MARKDOWN_FFI_PATH"); path != "" {
		return loadFFIFromPath(path)
	}
	if os.Getenv("HTML_TO_MARKDOWN_FFI_DISABLE_DOWNLOAD") != "" {
		return errors.New("html-to-markdown FFI download disabled and no library path provided")
	}
	version := os.Getenv("HTML_TO_MARKDOWN_FFI_VERSION")
	if version == "" {
		version = defaultFFIVersion
	}
	platform, archiveExt, libName, err := resolveFFIPlatform()
	if err != nil {
		return err
	}
	cacheDir, err := resolveCacheDir(version, platform)
	if err != nil {
		return err
	}
	libPath := filepath.Join(cacheDir, libName)
	if _, err := os.Stat(libPath); err == nil {
		return loadFFIFromPath(libPath)
	}
	archiveName := fmt.Sprintf("html-to-markdown-ffi-%s-%s.%s", version, platform, archiveExt)
	downloadURL := fmt.Sprintf("https://github.com/%s/releases/download/v%s/%s", githubRepo, version, archiveName)
	archivePath := filepath.Join(cacheDir, archiveName)

	if err := os.MkdirAll(cacheDir, 0o755); err != nil {
		return fmt.Errorf("create FFI cache dir: %w", err)
	}
	if err := downloadFile(downloadURL, archivePath); err != nil {
		return err
	}
	if err := extractArchive(archivePath, cacheDir); err != nil {
		return err
	}
	return loadFFIFromPath(libPath)
}

func loadFFIFromPath(path string) error {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))
	if ok := C.html_to_markdown_ffi_load(cPath); !bool(ok) {
		return fmt.Errorf("failed to load html-to-markdown FFI from %s", path)
	}
	return nil
}

func resolveFFIPlatform() (platform string, archiveExt string, libraryName string, err error) {
	switch runtime.GOOS {
	case "linux":
		switch runtime.GOARCH {
		case archAMD64:
			return "linux-x64", archiveTarGz, "libhtml_to_markdown_ffi.so", nil
		case archARM64:
			return "linux-arm64", archiveTarGz, "libhtml_to_markdown_ffi.so", nil
		}
	case "darwin":
		switch runtime.GOARCH {
		case archAMD64:
			return "darwin-x64", archiveTarGz, "libhtml_to_markdown_ffi.dylib", nil
		case archARM64:
			return "darwin-arm64", archiveTarGz, "libhtml_to_markdown_ffi.dylib", nil
		}
	case "windows":
		if runtime.GOARCH == archAMD64 {
			return "windows-x64", "zip", "html_to_markdown_ffi.dll", nil
		}
	}
	return "", "", "", fmt.Errorf("unsupported platform: %s/%s", runtime.GOOS, runtime.GOARCH)
}

func resolveCacheDir(version string, platform string) (string, error) {
	if dir := os.Getenv("HTML_TO_MARKDOWN_FFI_CACHE_DIR"); dir != "" {
		return filepath.Join(dir, version, platform), nil
	}
	baseDir, err := os.UserCacheDir()
	if err != nil {
		return "", fmt.Errorf("resolve user cache dir: %w", err)
	}
	return filepath.Join(baseDir, "html-to-markdown", "ffi", version, platform), nil
}

func downloadFile(url string, dest string) error {
	client := &http.Client{Timeout: 120 * time.Second}
	resp, err := client.Get(url)
	if err != nil {
		return fmt.Errorf("download FFI archive: %w", err)
	}
	defer func() {
		if err := resp.Body.Close(); err != nil {
			_ = err
		}
	}()
	if resp.StatusCode < 200 || resp.StatusCode > 299 {
		return fmt.Errorf("download FFI archive: unexpected status %s", resp.Status)
	}
	tmpPath := dest + ".tmp"
	out, err := os.Create(tmpPath)
	if err != nil {
		return fmt.Errorf("create archive file: %w", err)
	}
	if _, err := io.Copy(out, resp.Body); err != nil {
		if closeErr := out.Close(); closeErr != nil {
			return fmt.Errorf("write archive file: %w", closeErr)
		}
		return fmt.Errorf("write archive file: %w", err)
	}
	if err := out.Close(); err != nil {
		return fmt.Errorf("close archive file: %w", err)
	}
	if err := os.Rename(tmpPath, dest); err != nil {
		return fmt.Errorf("finalize archive file: %w", err)
	}
	return nil
}

func extractArchive(archivePath string, destDir string) error {
	if strings.HasSuffix(archivePath, ".zip") {
		return extractZip(archivePath, destDir)
	}
	if strings.HasSuffix(archivePath, ".tar.gz") {
		return extractTarGz(archivePath, destDir)
	}
	return fmt.Errorf("unsupported archive format: %s", archivePath)
}

func extractZip(path string, destDir string) error {
	reader, err := zip.OpenReader(path)
	if err != nil {
		return fmt.Errorf("open zip: %w", err)
	}
	defer func() {
		if err := reader.Close(); err != nil {
			_ = err
		}
	}()
	for _, file := range reader.File {
		if file.FileInfo().IsDir() {
			continue
		}
		if err := writeArchiveFile(file.Open, file.Name, file.Mode(), destDir); err != nil {
			return err
		}
	}
	return nil
}

func extractTarGz(path string, destDir string) error {
	file, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("open tar.gz: %w", err)
	}
	defer func() {
		if err := file.Close(); err != nil {
			_ = err
		}
	}()
	gzReader, err := gzip.NewReader(file)
	if err != nil {
		return fmt.Errorf("open gzip: %w", err)
	}
	defer func() {
		if err := gzReader.Close(); err != nil {
			_ = err
		}
	}()
	tarReader := tar.NewReader(gzReader)
	for {
		header, err := tarReader.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("read tar: %w", err)
		}
		if header.FileInfo().IsDir() {
			continue
		}
		if err := writeArchiveFile(func() (io.ReadCloser, error) {
			return io.NopCloser(tarReader), nil
		}, header.Name, header.FileInfo().Mode(), destDir); err != nil {
			return err
		}
	}
	return nil
}

func writeArchiveFile(open func() (io.ReadCloser, error), name string, mode os.FileMode, destDir string) error {
	reader, err := open()
	if err != nil {
		return fmt.Errorf("open archive entry: %w", err)
	}
	defer func() {
		if err := reader.Close(); err != nil {
			_ = err
		}
	}()

	targetPath := filepath.Join(destDir, filepath.Base(name))
	perm := mode.Perm()
	if perm == 0 {
		perm = 0o644
	}
	out, err := os.OpenFile(targetPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, perm)
	if err != nil {
		return fmt.Errorf("write archive entry: %w", err)
	}
	if _, err := io.Copy(out, reader); err != nil {
		if closeErr := out.Close(); closeErr != nil {
			return fmt.Errorf("write archive entry: %w", closeErr)
		}
		return fmt.Errorf("write archive entry: %w", err)
	}
	if err := out.Close(); err != nil {
		return fmt.Errorf("close archive entry: %w", err)
	}
	return nil
}
