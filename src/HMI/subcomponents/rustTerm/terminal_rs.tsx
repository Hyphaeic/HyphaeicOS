import { createSignal, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./terminal_rs.css";

interface TerminalRSProps {
    windowId: string;
}

/**
 * TerminalRS - Minimal PTY terminal component
 * 
 * Features:
 * - Dynamically resizes to fill parent container
 * - Spawns PTY session on mount
 * - Displays retro system status banner on startup
 * - Handles keyboard input forwarding to PTY
 * - Polls for PTY output and displays it
 */
export default function TerminalRS(props: TerminalRSProps) {
    const [output, setOutput] = createSignal<string>("");
    const [sessionId, setSessionId] = createSignal<string | null>(null);
    const [isReady, setIsReady] = createSignal(false);

    let containerRef: HTMLDivElement | undefined;
    let inputRef: HTMLTextAreaElement | undefined;
    let pollIntervalId: number | undefined;

    // Generate a unique session ID for this specific component instance
    // This ensures that even if the component is remounted quickly (HMR),
    // it treats previous sessions as distinct from new ones.
    const [mySessionId] = createSignal(`${props.windowId}-${Date.now()}`);

    // Character dimensions (approximate for monospace)
    const CHAR_WIDTH = 8.4;  // px
    const CHAR_HEIGHT = 16;  // px

    // Calculate terminal dimensions from container size
    const updateTerminalSize = () => {
        if (!containerRef) return;

        const rect = containerRef.getBoundingClientRect();
        const cols = Math.floor(rect.width / CHAR_WIDTH);
        const rows = Math.floor(rect.height / CHAR_HEIGHT);

        if (cols > 0 && rows > 0) {
            // Resize PTY if session exists
            const sid = sessionId();
            if (sid) {
                invoke("pty_resize", {
                    sessionId: sid,
                    rows: rows,
                    cols: cols,
                }).catch(console.error);
            }
        }
    };

    // Spawn PTY session and display banner
    const initializeTerminal = async () => {
        try {
            // Use our unique session ID
            const sid = mySessionId();

            // Get the system banner first
            const banner = await invoke<string>("get_system_banner", { sessionId: sid });
            setOutput(banner);

            // Spawn the PTY session
            await invoke("pty_spawn", { sessionId: sid });
            setSessionId(sid);
            setIsReady(true);

            // Initial resize
            updateTerminalSize();

            // Start polling for output
            startOutputPolling(sid);

        } catch (error) {
            console.error("[TerminalRS] Failed to initialize:", error);
            setOutput(prev => prev + `\n[ERROR] Failed to initialize terminal: ${error}`);
        }
    };

    // Poll for PTY output
    const startOutputPolling = (sid: string) => {
        pollIntervalId = window.setInterval(async () => {
            try {
                const data = await invoke<string>("pty_read", { sessionId: sid });
                if (data && data.length > 0) {
                    setOutput(prev => prev + data);

                    // Auto-scroll to bottom
                    if (containerRef) {
                        containerRef.scrollTop = containerRef.scrollHeight;
                    }
                }
            } catch (error) {
                // Session might be closed, stop polling
                if (pollIntervalId) {
                    clearInterval(pollIntervalId);
                }
            }
        }, 100); // Poll every 100ms
    };

    // Handle keyboard input
    const handleKeyDown = async (e: KeyboardEvent) => {
        const sid = sessionId();
        if (!sid || !isReady()) return;

        e.preventDefault();
        e.stopPropagation();

        let data = "";

        // Handle special keys
        switch (e.key) {
            case "Enter":
                data = "\r";
                break;
            case "Backspace":
                data = "\x7f";
                break;
            case "Tab":
                data = "\t";
                break;
            case "Escape":
                data = "\x1b";
                break;
            case "ArrowUp":
                data = "\x1b[A";
                break;
            case "ArrowDown":
                data = "\x1b[B";
                break;
            case "ArrowRight":
                data = "\x1b[C";
                break;
            case "ArrowLeft":
                data = "\x1b[D";
                break;
            default:
                // Handle Ctrl+C, Ctrl+D, etc.
                if (e.ctrlKey && e.key.length === 1) {
                    const code = e.key.toUpperCase().charCodeAt(0) - 64;
                    if (code >= 0 && code <= 31) {
                        data = String.fromCharCode(code);
                    }
                } else if (e.key.length === 1) {
                    data = e.key;
                }
        }

        if (data) {
            try {
                await invoke("pty_write", { sessionId: sid, data });
            } catch (error) {
                console.error("[TerminalRS] Write error:", error);
            }
        }
    };

    // Setup resize observer
    onMount(() => {
        initializeTerminal();

        // Observe container size changes
        const resizeObserver = new ResizeObserver(() => {
            updateTerminalSize();
        });

        if (containerRef) {
            resizeObserver.observe(containerRef);
        }

        // Focus input on mount
        setTimeout(() => {
            inputRef?.focus();
        }, 100);

        onCleanup(() => {
            resizeObserver.disconnect();

            // Stop polling
            if (pollIntervalId) {
                clearInterval(pollIntervalId);
            }

            // Close PTY session - use the unique ID we generated
            // We do this unconditionally to ensure we don't leak sessions in backend
            // The backend's reference counting will handle safe closure
            invoke("pty_close", { sessionId: mySessionId() }).catch(console.error);
        });
    });

    // Focus handling - ensure terminal captures input when clicked
    const handleContainerClick = () => {
        inputRef?.focus();
    };

    return (
        <div
            ref={containerRef}
            class="terminal-rs"
            onClick={handleContainerClick}
        >
            {/* Hidden textarea to capture keyboard input */}
            <textarea
                ref={inputRef}
                class="terminal-input"
                onKeyDown={handleKeyDown}
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck={false}
            />

            {/* Terminal output display */}
            <pre class="terminal-output">
                {output()}
                <span class="terminal-cursor">█</span>
            </pre>
        </div>
    );
}
