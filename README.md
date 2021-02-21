# tabcat
tabcat is an experimental project to send tablet events over a socket as a way to bring more modern graphics tablet capability to Wine experimentally. The goal is to develop a server that receives XInput2 events, translates them to a simple but reasonably complete intermediate form, serves these events over a socket, which the client can receive and translate into events.

## Server Design
The server design is simple. One thread is ran for the input driver, currently just X11. The X11 driver polls for XI2 events on the root window. These will be translated into the intermediate form in the X11 driver, and then sent over an MPSC channel (used in an SPSC form) to the host thread, which will serve it over a socket (probably a TCP socket, just to avoid any Wine specific hacks like calling Linux syscalls from within Wine.)

The server design will most likely be simple. It should guarantee some invariants on top of what the underlying windowing systems provide. For example, for convenience device IDs should probably be translated into IDs that are unique. (In XInput2, device IDs can be reused immediately.)

## Client Design
The client needs to be more complicated. For Win32, the client needs to be a library that can be loaded into the address space. At the entrypoint, this library needs to connect to the server to receive events. Several Win32 API functions will need to be patched:

* `PeekMessage`: Inject `WM_POINTER` events. We should 'coalesce' all contiguous motion events for one device currently pending, but otherwise play back the events otherwise in normal order.
* `GetMessage`: Similar to `PeekMessage`, but might be trickier since it blocks. It should send `WM_POINTER` events if they are pending.
* `DefWindowProc`: Translate `WM_POINTER` events into `WM_MOUSE` events. This allows them to be handled by the app. We may need to somehow prevent the XI2 event from being translated into an X11 mouse event to avoid duplicate events; maybe a Grab would be appropriate.
* `GetPointerCursorId`: Not sure yet. Maybe
* `GetPointer{Frame,}{,Pen,Touch}Info{,History}`: Implement these to handle providing coalesced events.
* `GetPointerType`: Override to return `PT_PEN`/`PT_TOUCH` as needed
* `EnableMouseInPointer`: This would be challenging, but should translate `WM_MOUSE` messages into `WM_POINTER` messages.
* `IsMouseInPointerEnabled`: Return true if the EnableMouseInPointer latch was set.
* `GetSystemMetrics`: Respond 1 to SM_TABLETPC so that applications will present the option to enable `WM_POINTER` mode. This doesn't really make that much sense.

Likely approach will be to construct trampolines, probably using the `detour` crate. As for injecting our DLL into the process, it will likely work as follows:

1. The process is created suspended.

2. An infinite jmp is placed at the PE entrypoint.

3. The process is resumed.

4. The main thread is polled to find when the EIP becomes stuck at the infinite loop.

5. The main thread is suspended.

6. The old entrypoint is written back.

7. We remotely allocate memory and write the path to our library into it.

8. A new thread in the process is remotely created at the address for LoadLibraryA using the address of the path.

9. Finally, once initialization is completed, the main thread can be resumed.

Some commercial applications may have anti-tampering mechanisms, so it may be necessary to use a more stealthy approach, or at least cover the tracks better. There are also some general DLL injector tools that may do the job better.
