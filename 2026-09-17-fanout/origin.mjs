// A stand-in origin that answers in GRIP. It tells the proxy to hold every
// request open as an HTTP stream, subscribed to the channel "test".
import http from "node:http";

http
  .createServer((req, res) => {
    const signed = req.headers["grip-sig"] ? "present" : "absent";
    console.log(`${req.method} ${req.url} (Grip-Sig ${signed})`);
    res.writeHead(200, {
      "Content-Type": "text/plain",
      "Grip-Hold": "stream",
      "Grip-Channel": "test",
    });
    res.end("[subscribed to channel test]\n");
  })
  .listen(3000, "127.0.0.1", () => console.log("GRIP origin on http://127.0.0.1:3000"));
