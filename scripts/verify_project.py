#!/usr/bin/env python3
"""
KolibriOS AI - Project Verification Script

Verifies the project state after all fixes are applied.
Generates a comprehensive report of the project status.
"""

import os
import sys
import json
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Tuple

class ProjectVerifier:
    """Verifies the KolibriOS AI project state."""
    
    def __init__(self, project_root: str = "/home/z/my-project"):
        self.project_root = Path(project_root)
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "checks": [],
            "files": {},
            "issues": [],
            "warnings": [],
            "summary": {}
        }
    
    def run_all_checks(self) -> Dict:
        """Run all verification checks."""
        print("=" * 60)
        print("KolibriOS AI - Project Verification")
        print("=" * 60)
        
        # Check 1: Workspace structure
        self._check_workspace_structure()
        
        # Check 2: Cargo.toml files
        self._check_cargo_files()
        
        # Check 3: Rust source files
        self._check_rust_sources()
        
        # Check 4: Python source files
        self._check_python_sources()
        
        # Check 5: Documentation
        self._check_documentation()
        
        # Check 6: Tests
        self._check_tests()
        
        # Generate summary
        self._generate_summary()
        
        return self.results
    
    def _check_workspace_structure(self) -> None:
        """Check workspace directory structure."""
        print("\n[CHECK] Workspace Structure")
        
        required_dirs = [
            "kernel", "vm", 
            "cells/memory_cell", "cells/processor_cell", "cells/protocols",
            "cells/io_cell", "cells/network_cell", "cells/process_cell", "cells/ai_cell",
            "koli_lang/compiler", "koli_lang/runtime",
            "unified_ai_agent/core",
            "apps/gui", "apps/file_manager", "apps/creative_assistant",
            "docs", "tests", "scripts"
        ]
        
        for dir_path in required_dirs:
            full_path = self.project_root / dir_path
            status = "✓" if full_path.exists() else "✗"
            print(f"  {status} {dir_path}")
            self.results["checks"].append({
                "check": "directory",
                "path": dir_path,
                "status": "pass" if full_path.exists() else "fail"
            })
    
    def _check_cargo_files(self) -> None:
        """Check Cargo.toml files for valid configuration."""
        print("\n[CHECK] Cargo.toml Files")
        
        cargo_files = list(self.project_root.rglob("Cargo.toml"))
        
        for cargo_file in cargo_files:
            try:
                content = cargo_file.read_text()
                rel_path = str(cargo_file.relative_to(self.project_root))
                
                issues = []
                
                # Check for thiserror in no_std crates
                if "#![no_std]" in self._get_lib_content(cargo_file.parent):
                    if "thiserror" in content:
                        issues.append("thiserror in no_std crate")
                
                # Check for valid TOML syntax
                if "[package]" not in content and "[workspace]" not in content:
                    issues.append("Invalid TOML - missing package/workspace section")
                
                status = "✓" if not issues else "⚠"
                print(f"  {status} {rel_path}")
                
                if issues:
                    self.results["warnings"].append({
                        "file": rel_path,
                        "issues": issues
                    })
                
                self.results["files"][rel_path] = {
                    "type": "cargo",
                    "status": "ok" if not issues else "warning",
                    "issues": issues
                }
                
            except Exception as e:
                print(f"  ✗ {cargo_file.relative_to(self.project_root)} - Error: {e}")
                self.results["issues"].append(str(e))
    
    def _get_lib_content(self, crate_dir: Path) -> str:
        """Get the content of lib.rs or main.rs in a crate."""
        lib_rs = crate_dir / "src" / "lib.rs"
        main_rs = crate_dir / "src" / "main.rs"
        
        if lib_rs.exists():
            return lib_rs.read_text()
        elif main_rs.exists():
            return main_rs.read_text()
        return ""
    
    def _check_rust_sources(self) -> None:
        """Check Rust source files for common issues."""
        print("\n[CHECK] Rust Source Files")
        
        rust_files = list(self.project_root.rglob("*.rs"))
        no_std_count = 0
        error_count = 0
        
        for rust_file in rust_files[:50]:  # Check first 50 files
            try:
                content = rust_file.read_text()
                rel_path = str(rust_file.relative_to(self.project_root))
                
                is_no_std = "#![no_std]" in content
                has_thiserror = "thiserror::Error" in content
                
                if is_no_std:
                    no_std_count += 1
                    if has_thiserror:
                        print(f"  ✗ {rel_path} - thiserror in no_std!")
                        error_count += 1
                        self.results["issues"].append({
                            "file": rel_path,
                            "issue": "thiserror in no_std crate",
                            "severity": "critical"
                        })
            
            except Exception as e:
                pass
        
        print(f"  Found {no_std_count} no_std crates")
        print(f"  Found {error_count} critical errors")
    
    def _check_python_sources(self) -> None:
        """Check Python source files."""
        print("\n[CHECK] Python Source Files")
        
        python_files = list(self.project_root.rglob("*.py"))
        valid_count = 0
        
        for py_file in python_files[:30]:
            try:
                content = py_file.read_text()
                compile(content, str(py_file), 'exec')
                valid_count += 1
            except SyntaxError as e:
                rel_path = str(py_file.relative_to(self.project_root))
                print(f"  ✗ {rel_path} - Syntax error: {e}")
                self.results["warnings"].append({
                    "file": rel_path,
                    "issue": f"Syntax error: {e}"
                })
        
        print(f"  {valid_count}/{len(python_files)} files have valid syntax")
    
    def _check_documentation(self) -> None:
        """Check documentation files."""
        print("\n[CHECK] Documentation")
        
        doc_files = list(self.project_root.glob("docs/**/*.md"))
        doc_files.extend(list(self.project_root.glob("*.md")))
        
        key_docs = ["README.md", "BUILD_LOG.md", "RELEASE_NOTES.md"]
        
        for doc in key_docs:
            doc_path = self.project_root / doc
            status = "✓" if doc_path.exists() else "✗"
            print(f"  {status} {doc}")
        
        print(f"  Total documentation files: {len(doc_files)}")
    
    def _check_tests(self) -> None:
        """Check test files."""
        print("\n[CHECK] Test Files")
        
        test_dirs = [
            "kernel/tests",
            "cells/memory_cell/tests",
            "cells/processor_cell/tests",
            "koli_lang/compiler/tests",
            "tests/functional"
        ]
        
        total_tests = 0
        for test_dir in test_dirs:
            test_path = self.project_root / test_dir
            if test_path.exists():
                test_files = list(test_path.glob("*.rs")) + list(test_path.glob("*.py"))
                count = len(test_files)
                total_tests += count
                print(f"  ✓ {test_dir}: {count} test files")
            else:
                print(f"  ✗ {test_dir}: missing")
        
        print(f"  Total test files: {total_tests}")
    
    def _generate_summary(self) -> None:
        """Generate summary of verification results."""
        print("\n" + "=" * 60)
        print("VERIFICATION SUMMARY")
        print("=" * 60)
        
        passed = sum(1 for c in self.results["checks"] if c["status"] == "pass")
        total = len(self.results["checks"])
        
        print(f"\nChecks: {passed}/{total} passed")
        print(f"Issues: {len(self.results['issues'])} critical")
        print(f"Warnings: {len(self.results['warnings'])} total")
        
        self.results["summary"] = {
            "checks_passed": passed,
            "checks_total": total,
            "issues_count": len(self.results["issues"]),
            "warnings_count": len(self.results["warnings"]),
            "status": "PASS" if not self.results["issues"] else "NEEDS_ATTENTION"
        }
        
        # Save results
        output_path = self.project_root / "docs" / "analysis" / "verification_results.json"
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w") as f:
            json.dump(self.results, f, indent=2, default=str)
        
        print(f"\nResults saved to: {output_path}")


def main():
    verifier = ProjectVerifier()
    results = verifier.run_all_checks()
    
    # Exit with error code if there are critical issues
    if results["issues"]:
        sys.exit(1)
    return 0


if __name__ == "__main__":
    main()
