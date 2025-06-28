#!/usr/bin/env python3
"""
Test script for submitting error reports to the Rust error reporting server.
Compatible with the original Yocto error reporting tool format.

Usage: ./test-send-error.py <server_url> <payload_file>
Example: ./test-send-error.py http://localhost:8000/ClientPost/JSON/ test-payload.json
"""

import sys
import json
import requests
import argparse
from datetime import datetime

def load_payload(file_path):
    """Load JSON payload from file."""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: Payload file '{file_path}' not found.")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in payload file: {e}")
        sys.exit(1)

def submit_error_report(server_url, payload):
    """Submit error report to the server."""
    headers = {
        'Content-Type': 'application/json',
        'User-Agent': 'Yocto Error Reporter Test Script'
    }

    try:
        print(f"Submitting error report to: {server_url}")
        print(f"Payload size: {len(json.dumps(payload))} bytes")

        response = requests.post(server_url, json=payload, headers=headers, timeout=30)

        print(f"Response status: {response.status_code}")

        if response.status_code == 200:
            result = response.json()
            print("✅ Error report submitted successfully!")
            print(f"Error ID: {result.get('id')}")
            print(f"View URL: {result.get('url')}")
            return True
        else:
            print("❌ Error submission failed!")
            print(f"Response: {response.text}")
            return False

    except requests.exceptions.RequestException as e:
        print(f"❌ Network error: {e}")
        return False
    except json.JSONDecodeError:
        print("❌ Invalid JSON response from server")
        return False

def create_sample_payload():
    """Create a sample payload for testing."""
    return {
        "machine": "qemux86-64",
        "distro": "poky",
        "distro_version": "4.0.15",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-22.04",
        "target_sys": "x86_64-poky-linux",
        "failure_task": "do_compile",
        "failure_package": "example-package",
        "error_type": "CompilationError",
        "error_details": f"Test error submitted at {datetime.now().isoformat()}\\n\\nERROR: example-package-1.0-r0 do_compile: oe_runmake failed\\nERROR: example-package-1.0-r0 do_compile: Execution of '/path/to/temp/run.do_compile.12345' failed with exit code 1:",
        "log_data": f"Build log for test error {datetime.now().isoformat()}\\n\\nNOTE: Started PRServer with DBfile: /path/to/build/cache/prserv.sqlite3, Address: 127.0.0.1:46175, PID: 12345\\nLoading cache: 100% |########################################| Time: 0:00:01\\nLoaded 1234 entries from dependency cache.\\nNOTE: Resolving any missing task queue dependencies\\n\\nBuild Configuration:\\nBB_VERSION        = \\"2.0.0\\"\\nBUILD_SYS         = \\"x86_64-linux\\"\\nNATIVELSBSTRING   = \\"ubuntu-22.04\\"\\nTARGET_SYS        = \\"x86_64-poky-linux\\"\\nMACHINE           = \\"qemux86-64\\"\\nDISTRO            = \\"poky\\"\\nDISTRO_VERSION    = \\"4.0.15\\"\\nTUNE_FEATURES     = \\"m64 core2\\"\\nTARGET_FPU        = \\"\\"\\nmeta              \\nmeta-poky         \\nmeta-yocto-bsp    = \\"test-branch:abc123def456789\\"\\n\\nNOTE: Fetching tasks for recipe example-package\\nERROR: example-package-1.0-r0 do_compile: oe_runmake failed\\nERROR: Logfile of failure stored in: /path/to/temp/work/core2-64-poky-linux/example-package/1.0-r0/temp/log.do_compile.12345\\nLog data truncated\\nERROR example-package-1.0-r0 do_compile: Function failed: do_compile (log file is located at /path/to/temp/work/core2-64-poky-linux/example-package/1.0-r0/temp/log.do_compile.12345)\\nERROR: Task (/path/to/recipes/example-package/example-package_1.0.bb:do_compile) failed with exit code '1'",
        "submitter_name": "Test User",
        "submitter_email": "test@example.com",
        "branch_commit": "test-branch:abc123def456789abcdef",
        "build_configuration": {
            "bb_version": "2.0.0",
            "tune_features": "m64 core2",
            "target_fpu": "",
            "meta_layers": [
                {
                    "name": "meta",
                    "path": "/path/to/poky/meta",
                    "commit": "abc123def456789",
                    "branch": "test-branch"
                },
                {
                    "name": "meta-poky",
                    "path": "/path/to/poky/meta-poky",
                    "commit": "abc123def456789",
                    "branch": "test-branch"
                },
                {
                    "name": "meta-yocto-bsp",
                    "path": "/path/to/poky/meta-yocto-bsp",
                    "commit": "abc123def456789",
                    "branch": "test-branch"
                }
            ]
        }
    }

def main():
    parser = argparse.ArgumentParser(description='Test error report submission to Yocto Error Reporting Server')
    parser.add_argument('server_url', help='Server URL (e.g., http://localhost:8000/ClientPost/JSON/)')
    parser.add_argument('payload_file', nargs='?', help='JSON payload file (optional, will create sample if not provided)')
    parser.add_argument('--create-sample', action='store_true', help='Create a sample payload file and exit')

    args = parser.parse_args()

    if args.create_sample:
        sample_payload = create_sample_payload()
        with open('sample-payload.json', 'w') as f:
            json.dump(sample_payload, f, indent=2)
        print("✅ Created sample-payload.json")
        return

    if args.payload_file:
        payload = load_payload(args.payload_file)
    else:
        print("No payload file provided, using sample payload...")
        payload = create_sample_payload()

    success = submit_error_report(args.server_url, payload)
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()
