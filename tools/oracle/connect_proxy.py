#!/usr/bin/env python3
"""Tiny HTTP CONNECT proxy. The Docker build container has broken IPv6 and the Haskell
HTTP stack does not fall back to IPv4, so we tunnel through the host's network stack."""
import asyncio, sys

async def pipe(r, w):
    try:
        while data := await r.read(65536):
            w.write(data); await w.drain()
    except Exception:
        pass
    finally:
        try: w.close()
        except Exception: pass

async def handle(cr, cw):
    try:
        head = await cr.readuntil(b"\r\n\r\n")
        line = head.split(b"\r\n")[0].decode()
        method, target, _ = line.split(" ", 2)
        if method != "CONNECT":
            cw.write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n"); await cw.drain(); cw.close(); return
        host, port = target.rsplit(":", 1)
        ur, uw = await asyncio.open_connection(host, int(port))
        cw.write(b"HTTP/1.1 200 Connection established\r\n\r\n"); await cw.drain()
        await asyncio.gather(pipe(cr, uw), pipe(ur, cw))
    except Exception as e:
        try: cw.close()
        except Exception: pass

async def main(port):
    srv = await asyncio.start_server(handle, "0.0.0.0", port)
    async with srv:
        await srv.serve_forever()

asyncio.run(main(int(sys.argv[1]) if len(sys.argv) > 1 else 3128))
