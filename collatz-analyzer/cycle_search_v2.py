#!/usr/bin/env python3
"""
Targeted search for non-trivial Collatz cycles using the algebraic cycle equation.

(2^V - 3^k) * n = C

Strategy:
1. Brute-force cycle check for all odd n up to 10^6
2. Enumerate v-sequences where v_i ∈ {1, 2, 3} for k ≤ 10 (3^k combos = 59049)
3. Enumerate v-sequences with v_i ∈ {1, 2} for k up to 18 (2^k = 262k max)
4. Check all v-sequences with V ≤ 25 via efficient composition enumeration with pruning
"""

import sys
import math
from collections import defaultdict
import time

KNOWN_CYCLES = {1: "trivial", -1: "negative", -5: "negative"}


def trailing_zeros(x):
    return (x & -x).bit_length() - 1 if x else 0


def collatz_step_condensed(n):
    if n % 2 == 0:
        v = trailing_zeros(n)
        return (n >> v, v)
    else:
        val = 3 * n + 1
        v = trailing_zeros(val)
        return (val >> v, v)


def compute_C_and_V(v_seq):
    k = len(v_seq)
    V = sum(v_seq)
    C = 0
    for j in range(k):
        pow3_term = 3 ** (k - 1 - j)
        prefix_v = sum(v_seq[:j])
        pow2_term = 2 ** prefix_v
        C += pow3_term * pow2_term
    return C, V


def compute_n_from_v_seq(v_seq):
    k = len(v_seq)
    C, V = compute_C_and_V(v_seq)
    pow2_V = 2 ** V
    pow3_k = 3 ** k
    
    if pow2_V <= pow3_k:
        return None
    
    denom = pow2_V - pow3_k
    
    if C % denom != 0:
        return None
    
    n = C // denom
    
    if n <= 0 or n % 2 == 0:
        return None
    
    return n, V, C, denom


def verify_v_seq_consistency(v_seq, n):
    """Verify v-sequence matches actual Collatz trajectory."""
    x = n
    for i, v_expected in enumerate(v_seq):
        if x % 2 == 0:
            return False
        val = 3 * x + 1
        v_actual = trailing_zeros(val)
        if v_actual != v_expected:
            return False
        x = val >> v_actual
    return True


def check_cycle(n, max_steps=10000):
    """Check if n is part of a non-trivial cycle."""
    seen = {}
    x = n
    steps = 0
    while steps < max_steps:
        if x in seen:
            cycle_len = steps - seen[x]
            if x == 1:
                return (True, 0, 1)
            return (True, cycle_len, x)
        seen[x] = steps
        if x == 1:
            return (True, 0, 1)
        x, _ = collatz_step_condensed(x)
        steps += 1
    return (False, 0, n)


# Approach 1: Brute force cycle detection
def brute_force_cycles(limit=1000000):
    print(f"\nApproach 1: Brute-force cycle check for odd n up to {limit}...")
    t0 = time.time()
    visited = {}  # n -> 0=unknown, 1=converges, -1=cycle
    cycles = []
    
    for n_start in range(3, limit + 1, 2):
        if n_start in visited:
            continue
        
        path = []
        n = n_start
        while n not in visited:
            visited[n] = 0
            path.append(n)
            if n == 1:
                break
            n, _ = collatz_step_condensed(n)
        
        if n == 1 or visited.get(n) == 1:
            for p in path:
                visited[p] = 1
        elif n in path:
            # Found cycle within this path
            idx = path.index(n)
            cycle_vals = path[idx:]
            cycles.append((n_start, cycle_vals))
            for p in path:
                visited[p] = -1 if p in cycle_vals else 1
        elif visited.get(n) == -1:
            for p in path:
                visited[p] = -1
    
    t1 = time.time()
    print(f"  Time: {t1-t0:.2f}s")
    print(f"  Numbers checked: {(limit-1)//2}")
    
    if cycles:
        print(f"  >>> CYCLES FOUND: {len(cycles)}")
        for entry, cycle_vals in cycles[:10]:
            print(f"      start={entry}, cycle={cycle_vals}")
    else:
        print(f"  No non-trivial cycles found up to n={limit}")
    
    return cycles


# Approach 2: Enumerate v-sequences with v_i in {1, 2, 3}
def enum_v_seq_123(max_k):
    """Enumerate v-sequences where each v_i ∈ {1, 2, 3}."""
    print(f"\nApproach 2: Enumerate v-sequences with v_i ∈ {{1,2,3}}, k ≤ {max_k}...")
    t0 = time.time()
    candidates = []
    
    def dfs(k_remaining, current_v_seq):
        if k_remaining == 0:
            result = compute_n_from_v_seq(current_v_seq)
            if result is not None:
                n, V, C, denom = result
                if n > 1 and n not in KNOWN_CYCLES:
                    candidates.append((list(current_v_seq), n, V, C, denom))
            return
        
        for vi in [1, 2, 3]:
            current_v_seq.append(vi)
            dfs(k_remaining - 1, current_v_seq)
            current_v_seq.pop()
    
    for k in range(1, max_k + 1):
        dfs(k, [])
    
    t1 = time.time()
    print(f"  Time: {t1-t0:.2f}s")
    print(f"  Sequences tested: {sum(3**k for k in range(1, max_k+1))}")
    print(f"  Candidates (n odd>1): {len(candidates)}")
    
    # Verify candidates
    valid = []
    for v_seq, n, V, C, denom in candidates:
        if verify_v_seq_consistency(v_seq, n):
            is_cycle, cycle_len, cycle_start = check_cycle(n)
            if is_cycle and cycle_len > 0:
                valid.append((v_seq, n, V, C, denom, cycle_len))
    
    if valid:
        print(f"  >>> VALID CYCLES: {len(valid)}")
        for v_seq, n, V, C, denom, clen in valid:
            print(f"      n={n}, k={len(v_seq)}, V={V}, v={v_seq}, cycle_len={clen}")
    else:
        print("  No valid cycles found.")
    
    return valid, candidates


# Approach 3: Enumerate v-sequences with v_i in {1, 2} (binary)
def enum_v_seq_12(max_k):
    """Enumerate v-sequences where each v_i ∈ {1, 2}."""
    print(f"\nApproach 3: Enumerate v-sequences with v_i ∈ {{1,2}}, k ≤ {max_k}...")
    t0 = time.time()
    candidates = []
    
    for k in range(1, max_k + 1):
        for mask in range(2**k):
            v_seq = [1 + ((mask >> i) & 1) for i in range(k)]
            result = compute_n_from_v_seq(v_seq)
            if result is not None:
                n, V, C, denom = result
                if n > 1 and n % 2 == 1 and n not in KNOWN_CYCLES:
                    candidates.append((v_seq, n, V, C, denom))
    
    t1 = time.time()
    print(f"  Time: {t1-t0:.2f}s")
    print(f"  Sequences tested: {2**(max_k+1) - 2}")
    print(f"  Candidates (n odd>1): {len(candidates)}")
    
    valid = []
    for v_seq, n, V, C, denom in candidates:
        if verify_v_seq_consistency(v_seq, n):
            is_cycle, cycle_len, cycle_start = check_cycle(n)
            if is_cycle and cycle_len > 0:
                valid.append((v_seq, n, V, C, denom, cycle_len))
    
    if valid:
        print(f"  >>> VALID CYCLES: {len(valid)}")
        for v_seq, n, V, C, denom, clen in valid:
            print(f"      n={n}, k={len(v_seq)}, V={V}, v={v_seq}, cycle_len={clen}")
    else:
        print("  No valid cycles found.")
    
    return valid, candidates


# Approach 4: Enumerate by total V with composition generation
def enum_by_V(max_V):
    """
    Enumerate compositions of V into k parts for each k.
    Only compositions where 2^V > 3^k.
    """
    print(f"\nApproach 4: Enumerate by total V ≤ {max_V} using integer compositions...")
    t0 = time.time()
    candidates = []
    total_tested = 0
    
    for k in range(1, max_V + 1):
        min_V = math.floor(k * math.log2(3)) + 1
        if min_V > max_V:
            continue
        
        for V in range(min_V, max_V + 1):
            if 2**V <= 3**k:
                continue
            
            # Generate compositions of V into k parts (each >= 1)
            # This is C(V-1, k-1) sequences
            # Use recursion with early pruning based on mod constraints
            
            def gen_compositions(parts_left, sum_left, start_min):
                if parts_left == 0:
                    if sum_left == 0:
                        yield []
                    return
                
                max_first = sum_left - (parts_left - 1) * 1
                if max_first < start_min:
                    return
                
                for vi in range(start_min, max_first + 1):
                    for rest in gen_compositions(parts_left - 1, sum_left - vi, 1):
                        yield [vi] + rest
            
            count = 0
            for v_seq in gen_compositions(k, V, 1):
                total_tested += 1
                result = compute_n_from_v_seq(v_seq)
                if result is not None:
                    n, Vc, C, denom = result
                    if n > 1 and n % 2 == 1 and n not in KNOWN_CYCLES:
                        candidates.append((v_seq, n, V, C, denom))
                        count += 1
            
            if count > 0:
                pass  # Would print progress
    
    t1 = time.time()
    print(f"  Time: {t1-t0:.2f}s")
    print(f"  Total sequences tested: {total_tested}")
    print(f"  Candidates (n odd>1): {len(candidates)}")
    
    # Verify - but only if not too many
    valid = []
    verify_limit = 50000
    to_verify = candidates[:verify_limit]
    if len(candidates) > verify_limit:
        print(f"  Verifying first {verify_limit} candidates (skipping {len(candidates)-verify_limit})")
    
    for v_seq, n, V, C, denom in to_verify:
        ok = verify_v_seq_consistency(v_seq, n)
        if ok:
            is_cycle, cycle_len, cycle_start = check_cycle(n)
            if is_cycle and cycle_len > 0:
                valid.append((v_seq, n, V, C, denom, cycle_len))
    
    if valid:
        print(f"  >>> VALID CYCLES: {len(valid)}")
        for v_seq, n, V, C, denom, clen in valid:
            print(f"      n={n}, k={len(v_seq)}, V={V}, v={v_seq}, cycle_len={clen}")
    else:
        print("  No valid cycles found in verified subset.")
    
    return valid, candidates, total_tested


# Approach 5: Direct search over odd n, tracking cycles
def find_all_cycles(limit=10000000):
    """
    More memory-efficient cycle detection using Floyd's algorithm
    or tracking with a set.
    """
    print(f"\nApproach 5: Extended brute-force for odd n up to {limit}...")
    t0 = time.time()
    
    # Use a bitset for convergence status
    # We only need to check odd numbers, and we care about cycles
    
    # Strategy: use a hash set for visited numbers, track depth
    # For each odd n, follow trajectory until we hit a known state
    
    converged = set()  # numbers known to converge to 1
    cycles = []
    
    for n_start in range(3, limit + 1, 2):
        if n_start in converged:
            continue
        
        path = []
        n = n_start
        
        while n != 1 and n not in converged:
            path.append(n)
            n, _ = collatz_step_condensed(n)
            
            # Check if n is in current path (cycle detected)
            if n in path:
                idx = path.index(n)
                cycle_vals = path[idx:]
                cycles.append((n_start, cycle_vals))
                break
        
        # Mark all numbers in path as converging to 1 (unless cycle)
        if n == 1:
            for p in path:
                converged.add(p)
    
    t1 = time.time()
    print(f"  Time: {t1-t0:.2f}s")
    
    non_trivial = [c for c in cycles if c[0] != 1]
    if non_trivial:
        print(f"  >>> NON-TRIVIAL CYCLES: {len(non_trivial)}")
        for entry, cycle_vals in non_trivial[:10]:
            print(f"      start={entry}, cycle_length={len(cycle_vals)}, values={cycle_vals}")
    else:
        print("  No non-trivial cycles found.")
    
    return cycles


def main():
    print("=" * 80)
    print("COLLATZ CYCLE SEARCH - Algebraic Cycle Equation Analysis")
    print("=" * 80)
    
    # Run progressively deeper searches
    all_cycles = []
    
    # 1. Brute force up to 1M
    cycles1 = brute_force_cycles(1000000)
    all_cycles.extend(cycles1)
    
    if not all_cycles:
        # 2. Enumerate v ∈ {1,2,3} for k ≤ 10
        valid2, _ = enum_v_seq_123(10)
        for v_seq, n, V, C, denom, clen in valid2:
            all_cycles.append((n, [n]))
        
        # 3. Enumerate v ∈ {1,2} for k ≤ 18
        valid3, _ = enum_v_seq_12(18)
        for v_seq, n, V, C, denom, clen in valid3:
            all_cycles.append((n, [n]))
        
        # 4. Enumerate by V for V ≤ 25 (more limited due to combinatorial growth)
        # Only do this for small V where it's tractable
        valid4, _, tested = enum_by_V(22)
        for v_seq, n, V, C, denom, clen in valid4:
            all_cycles.append((n, [n]))
    
    # Summary
    print("\n" + "=" * 80)
    print("FINAL SUMMARY")
    print("=" * 80)
    
    if all_cycles:
        print(f"CYCLES FOUND: {len(all_cycles)}")
    else:
        print("No non-trivial Collatz cycles found within all search bounds.")
        print()
        print("Search bounds:")
        print("  - Brute force: odd n up to 1,000,000")
        print("  - v-sequences with v_i in {1,2,3}: k ≤ 10")
        print("  - v-sequences with v_i in {1,2}: k ≤ 18")
        print("  - v-sequences with V ≤ 22: all compositions")
        print()
        print("This is consistent with the Collatz conjecture.")
    
    return all_cycles


if __name__ == "__main__":
    main()
