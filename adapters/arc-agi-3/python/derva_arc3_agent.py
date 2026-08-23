#!/usr/bin/env python3
# AUDIT-LENSES: Guido van Rossum, Linus Torvalds, Steve Jobs
# INVARIANT: Python ARC-AGI-3 Agent Adapter communicating with Authoritative Rust Subprocess via JSONL. No game heuristics or hardcoded logic.

import sys
import json
import subprocess
import os

class DervaArc3Agent:
    def __init__(self, rust_bin_path=None):
        if rust_bin_path is None:
            rust_bin_path = os.path.abspath(os.path.join(
                os.path.dirname(__file__), "..", "..", "..", "target", "release", "derva-arc3-adapter"
            ))
        
        # Start authoritative Rust adapter process
        self.proc = subprocess.Popen(
            [rust_bin_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        self.step_count = 0

    def step(self, observation_dict, action_space_dict):
        self.step_count += 1
        payload = {
            "step": self.step_count,
            "observation": observation_dict,
            "action_space": action_space_dict
        }
        json_req = json.dumps(payload) + "\n"
        self.proc.stdin.write(json_req)
        self.proc.stdin.flush()

        response_line = self.proc.stdout.readline()
        if not response_line:
            raise RuntimeError("DERVA Rust adapter process terminated unexpectedly!")
        
        resp = json.loads(response_line)
        return resp


    def close(self):
        if self.proc:
            self.proc.terminate()

if __name__ == "__main__":
    print("[DERVA ARC-AGI-3 Python Adapter initialized successfully]")
