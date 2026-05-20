import "@testing-library/jest-dom";

// Stub the Tauri IPC bridge so components that import @tauri-apps/api don't throw.
Object.defineProperty(window, "__TAURI_INTERNALS__", { value: { ipc: () => {} } });
