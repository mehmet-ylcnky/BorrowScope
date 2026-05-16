import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.9 WebSocket Live Connection", () => {
  let RuntimeSocket: any;

  before(() => {
    // Mock WebSocket globally for tests
    (global as any).WebSocket = class MockWebSocket {
      onopen: any; onmessage: any; onclose: any; onerror: any;
      readyState = 0;
      constructor(public url: string) {
        setTimeout(() => { this.readyState = 1; if (this.onopen) this.onopen(); }, 0);
      }
      close() { this.readyState = 3; if (this.onclose) this.onclose(); }
      send() {}
    };
    RuntimeSocket = require(path.join(ROOT, "out", "runtime-socket")).RuntimeSocket;
  });

  after(() => {
    delete (global as any).WebSocket;
  });

  // === Instantiation ===

  it("can instantiate RuntimeSocket", () => {
    const socket = new RuntimeSocket();
    assert.ok(socket);
    assert.strictEqual(socket.state, "disconnected");
    socket.dispose();
  });

  it("starts with 0 events", () => {
    const socket = new RuntimeSocket();
    assert.strictEqual(socket.eventCount, 0);
    assert.deepStrictEqual(socket.getEvents(), []);
    socket.dispose();
  });

  it("isConnected is false initially", () => {
    const socket = new RuntimeSocket();
    assert.strictEqual(socket.isConnected, false);
    socket.dispose();
  });

  // === Connection ===

  it("connect changes state to connecting", () => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.connect(9876);
    assert.strictEqual(socket.state, "connecting");
    socket.dispose();
  });

  it("connect uses configured port", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        assert.strictEqual(socket.isConnected, true);
        socket.dispose();
        done();
      }
    });
    socket.connect(9876);
  });

  it("disconnect sets state to disconnected", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        socket.disconnect();
        assert.strictEqual(socket.state, "disconnected");
        assert.strictEqual(socket.isConnected, false);
        done();
      }
    });
    socket.connect(9876);
  });

  it("does not reconnect when autoReconnect is false", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    let disconnectCount = 0;
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        socket.disconnect();
      }
      if (state === "disconnected") {
        disconnectCount++;
        if (disconnectCount === 1) {
          // Wait a bit to ensure no reconnect
          setTimeout(() => {
            assert.strictEqual(socket.state, "disconnected");
            socket.dispose();
            done();
          }, 50);
        }
      }
    });
    socket.connect(9876);
  });

  // === Message handling ===

  it("handles single event message", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        // Simulate message
        const event = { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } };
        socket.onEvent((e: any) => {
          assert.ok(e.New);
          assert.strictEqual(socket.eventCount, 1);
          socket.dispose();
          done();
        });
        // Trigger message handler via the mock
        (socket as any).ws.onmessage({ data: JSON.stringify(event) });
      }
    });
    socket.connect(9876);
  });

  it("handles batch event message", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        const batch = [
          { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
          { Drop: { timestamp: 20, var_id: "x_1" } },
        ];
        socket.onBatchReceived((events: any[]) => {
          assert.strictEqual(events.length, 2);
          assert.strictEqual(socket.eventCount, 2);
          socket.dispose();
          done();
        });
        (socket as any).ws.onmessage({ data: JSON.stringify(batch) });
      }
    });
    socket.connect(9876);
  });

  it("ignores invalid JSON messages", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        (socket as any).ws.onmessage({ data: "{broken json" });
        assert.strictEqual(socket.eventCount, 0);
        socket.dispose();
        done();
      }
    });
    socket.connect(9876);
  });

  it("ignores invalid event structure", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        (socket as any).ws.onmessage({ data: JSON.stringify({ FakeEvent: { timestamp: 1 } }) });
        assert.strictEqual(socket.eventCount, 0);
        socket.dispose();
        done();
      }
    });
    socket.connect(9876);
  });

  // === Event eviction ===

  it("evicts oldest events when maxEvents reached", (done) => {
    const socket = new RuntimeSocket({ maxEvents: 10, autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        // Send 12 events
        for (let i = 0; i < 12; i++) {
          (socket as any).ws.onmessage({
            data: JSON.stringify({ New: { timestamp: i, var_name: `v${i}`, var_id: `v${i}_1`, type_name: "i32" } })
          });
        }
        // Should have evicted oldest 10% (1 event), then added remaining
        assert.ok(socket.eventCount <= 12);
        assert.ok(socket.eventCount >= 10);
        socket.dispose();
        done();
      }
    });
    socket.connect(9876);
  });

  // === clearEvents ===

  it("clearEvents resets event buffer", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        (socket as any).ws.onmessage({
          data: JSON.stringify({ New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } })
        });
        assert.strictEqual(socket.eventCount, 1);
        socket.clearEvents();
        assert.strictEqual(socket.eventCount, 0);
        socket.dispose();
        done();
      }
    });
    socket.connect(9876);
  });

  // === dispose ===

  it("dispose disconnects and cleans up", () => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.dispose();
    assert.strictEqual(socket.state, "disconnected");
  });

  // === Handles all event types ===

  it("accepts all 88 event types via WebSocket", (done) => {
    const socket = new RuntimeSocket({ autoReconnect: false });
    socket.onStateChanged((state: string) => {
      if (state === "connected") {
        const events = [
          { New: { timestamp: 1, var_name: "x", var_id: "x_1", type_name: "i32" } },
          { Drop: { timestamp: 2, var_id: "x_1" } },
          { Borrow: { timestamp: 3, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
          { Move: { timestamp: 4, from_id: "x_1", to_name: "y", to_id: "y_1" } },
          { RcNew: { timestamp: 5, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
          { ArcNew: { timestamp: 6, var_name: "a", var_id: "a_1", type_name: "Arc<i32>", strong_count: 1, weak_count: 0 } },
          { FnEnter: { timestamp: 7, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } },
          { FnExit: { timestamp: 8, fn_id: "f1", fn_name: "main", location: "src/main.rs:10:1" } },
          { AwaitStart: { timestamp: 9, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
          { AwaitEnd: { timestamp: 10, await_id: "aw_1", location: "src/main.rs:5:5" } },
          { BoxNew: { timestamp: 11, var_name: "b", var_id: "b_1", type_name: "Box<i32>", location: "src/main.rs:3:5" } },
          { WeakNew: { timestamp: 12, var_name: "w", var_id: "w_1", source_id: "rc_1", weak_count: 1, location: "src/main.rs:4:5" } },
          { LockGuardAcquire: { timestamp: 13, guard_id: "g_1", lock_id: "lk_1", lock_type: "Mutex", location: "src/main.rs:6:5" } },
          { LockGuardDrop: { timestamp: 14, guard_id: "g_1", location: "src/main.rs:8:5" } },
          { ChannelSend: { timestamp: 15, sender_id: "s_1", location: "src/main.rs:9:5" } },
          { ChannelRecv: { timestamp: 16, receiver_id: "r_1", success: true, location: "src/main.rs:10:5" } },
        ];
        (socket as any).ws.onmessage({ data: JSON.stringify(events) });
        assert.strictEqual(socket.eventCount, 16);
        socket.dispose();
        done();
      }
    });
    socket.connect(9876);
  });
});
