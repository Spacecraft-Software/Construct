# Spacecraft Common Lisp Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Common Lisp systems programming. It provides complete, compile-checked skeletons for `lparallel` worker pools, `CFFI` pointer tracking, high-speed numeric calculations targeting SBCL, and unit testing.

---

## 1. Concurrency vs. Parallelism: Architectural Decisions

Common Lisp does not define threads in the ANSI standard, but Bordeaux-Threads is the standard compatibility layer. For multi-core scaling, OCaml-like event loops or Rust-like fork-join structures are handled using high-level libraries.

- **Parallelism (Compute-bound):** Handled via **lparallel**. It manages a queue of workers (operating on top of native OS threads). Use it to parallelize arrays, loops, and mappings.
- **Concurrency (Service Loops):** Handled via **Bordeaux-Threads** (`bt:make-thread`). Use it for long-running, coarse-grained background tasks (like logging servers, listener sockets, or job coordinators).
- **Atomics and CAS:** SBCL supports compare-and-swap natively (`sb-ext:compare-and-swap`). Use it for thread-safe lock-free collections (e.g. queues, stacks, shared counters) to avoid lock contention bottlenecks.

---

## 2. Parallel Compute Skeleton (lparallel)

Compute-heavy tasks should run in the `lparallel` kernel. Initialize the kernel once on program startup and size it to the number of physical cores.

```lisp
(in-package :cl-user)

(defpackage :telemetry-parallel
  (:use :cl)
  (:export #:parallel-sum-array #:init-compute-kernel #:shutdown-compute-kernel))

(in-package :telemetry-parallel)

;; Global kernel holder (unbound by default)
(defvar *compute-kernel* nil)

(defun init-compute-kernel (worker-count)
  "Initializes the lparallel kernel once on startup."
  (unless *compute-kernel*
    (setf *compute-kernel* (lparallel:make-kernel worker-count))
    (setf lparallel:*kernel* *compute-kernel*)))

(defun shutdown-compute-kernel ()
  "Shuts down the lparallel kernel cleanly on exit."
  (when *compute-kernel*
    (let ((kernel *compute-kernel*))
      (setf *compute-kernel* nil)
      (setf lparallel:*kernel* nil)
      (lparallel:end-kernel :kernel kernel))))

(defun parallel-sum-array (arr)
  "Sums an array of double-floats in parallel using lparallel."
  (declare (type (simple-array double-float (*)) arr)
           (optimize (speed 3) (safety 1)))
  (let ((len (length arr)))
    (if (< len 1000)
        ;; Fall back to serial fold for small arrays to avoid scheduling overhead
        (reduce #'+ arr)
        ;; Parallel reduce splits the vector ranges across the task queue
        (lparallel:preduce #'+ arr))))
```

---

## 3. CFFI Safe Pointer Mapping & GC Integration

To prevent foreign memory leaks, wrap C pointers in Lisp structures and register finalizers using the `trivial-garbage` library.

### C Struct Example
```c
// telemetry.h
typedef struct {
    double cpu_usage;
    int ram_usage;
} TelemetryData;
```

### Lisp FFI Wrapper (`telemetry-ffi.lisp`)
```lisp
(defpackage :telemetry-ffi
  (:use :cl)
  (:export #:telemetry-data
           #:make-telemetry-data
           #:telemetry-data-cpu
           #:telemetry-data-ram
           #:with-telemetry))

(in-package :telemetry-ffi)

;; Define foreign structure
(cffi:defcstruct telemetry-data-c
  (cpu-usage :double)
  (ram-usage :int))

;; Lisp wrapper class to hold the pointer
(defclass telemetry-data ()
  ((pointer :initarg :pointer :reader telemetry-data-pointer)))

(defun free-telemetry-pointer (ptr)
  "Callback for the garbage collector to release foreign memory."
  (unless (cffi:null-pointer-p ptr)
    (cffi:foreign-free ptr)))

(defun make-telemetry-data (cpu ram)
  "Allocates a foreign struct, registers it with the GC, and returns Lisp wrapper."
  (let* ((ptr (cffi:foreign-alloc '(:struct telemetry-data-c)))
         (wrapper (make-instance 'telemetry-data :pointer ptr)))
    (setf (cffi:foreign-slot-value ptr '(:struct telemetry-data-c) 'cpu-usage) (coerce cpu 'double-float)
          (cffi:foreign-slot-value ptr '(:struct telemetry-data-c) 'ram-usage) (coerce ram 'integer))
    ;; Register finalizer so that foreign-free is called when Lisp GC reclaims the wrapper
    (tg:finalize wrapper (lambda () (free-telemetry-pointer ptr)))
    wrapper))

(defun telemetry-data-cpu (data)
  (declare (type telemetry-data data))
  (cffi:foreign-slot-value (telemetry-data-pointer data) '(:struct telemetry-data-c) 'cpu-usage))

(defun telemetry-data-ram (data)
  (declare (type telemetry-data data))
  (cffi:foreign-slot-value (telemetry-data-pointer data) '(:struct telemetry-data-c) 'ram-usage))

(defmacro with-telemetry ((var cpu ram) &body body)
  "Allocates a foreign struct on the stack (dynamic extent) for local scope."
  `(cffi:with-foreign-object (,var '(:struct telemetry-data-c))
     (setf (cffi:foreign-slot-value ,var '(:struct telemetry-data-c) 'cpu-usage) (coerce ,cpu 'double-float)
           (cffi:foreign-slot-value ,var '(:struct telemetry-data-c) 'ram-usage) (coerce ,ram 'integer))
     ,@body))
```

---

## 4. High-Speed Numeric Loop & TCO (SBCL Optimized)

To generate optimized native code in SBCL, specify declarations and compilation flags, avoiding consing and generic arithmetic.

```lisp
(defpackage :numeric-compute
  (:use :cl)
  (:export #:factorial #:sum-recproc))

(in-package :numeric-compute)

(declaim (ftype (function (fixnum) fixnum) factorial))
(defun factorial (n)
  "Tail-recursive factorial with SBCL speed declarations."
  (declare (type fixnum n)
           (optimize (speed 3) (safety 1) (debug 1)))
  (labels ((fact-loop (x acc)
             (declare (type fixnum x acc))
             (if (<= x 1)
                 acc
                 (fact-loop (1- x) (the fixnum (* acc x))))))
    (fact-loop n 1)))

(defun sum-recproc (vec)
  "Optimized array traversal using in-place modification to avoid consing."
  (declare (type (simple-array double-float (*)) vec)
           (optimize (speed 3) (safety 1)))
  (let ((len (length vec))
        (sum 0.0d0))
    (declare (type double-float sum)
             (type fixnum len))
    (dotimes (i len sum)
      (setf sum (+ sum (aref vec i))))))
```

---

## 5. Testing: FiveAM

Use **FiveAM** to manage test suites. Treat testing warnings and regression coverage as part of the quality gate.

```lisp
;; test/test-suite.lisp

(defpackage :telemetry-tests
  (:use :cl :fiveam)
  (:export #:run-all-tests))

(in-package :telemetry-tests)

(def-suite telemetry-suite :description "System suite for telemetry Lisp components")
(in-suite telemetry-suite)

(test test-factorial
  "Tests the tail-recursive factorial function"
  (is (= 1 (numeric-compute:factorial 0)))
  (is (= 1 (numeric-compute:factorial 1)))
  (is (= 120 (numeric-compute:factorial 5))))

(test test-cffi-macro
  "Tests stack-allocation and value setting inside CFFI block"
  (telemetry-ffi:with-telemetry (ptr 0.85d0 4096)
    (is (cffi:pointerp ptr))
    (is (= 0.85d0 (cffi:foreign-slot-value ptr '(:struct telemetry-ffi::telemetry-data-c) 'telemetry-ffi::cpu-usage)))
    (is (= 4096 (cffi:foreign-slot-value ptr '(:struct telemetry-ffi::telemetry-data-c) 'telemetry-ffi::ram-usage)))))

(defun run-all-tests ()
  (let ((results (run 'telemetry-suite)))
    (explain! results)
    (results-status results)))
```

---

## 6. ASDF System Configuration (`telemetry.asd`)

ASDF manages dependencies and builds the project. Enforce compilation warning gates.

```lisp
;; telemetry.asd
(asdf:defsystem #:telemetry
  :description "Spacecraft Telemetry Service in Common Lisp"
  :author "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>"
  :license "GPL-3.0-or-later"
  :depends-on (#:bordeaux-threads #:lparallel #:cffi #:trivial-garbage)
  :components ((:file "telemetry-ffi")
               (:file "numeric-compute")
               (:file "telemetry-parallel"))
  :in-order-to ((asdf:test-op (asdf:test-op #:telemetry/tests))))

(asdf:defsystem #:telemetry/tests
  :depends-on (#:telemetry #:fiveam)
  :components ((:module "test"
                :components ((:file "test-suite"))))
  :perform (asdf:test-op (op c)
                         (uiop:symbol-call :telemetry-tests :run-all-tests)))
```

---

## 7. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **`safety 0` globally** | Unpredictable segmentation faults | Keep `safety 1` default. Only use `safety 0` locally inside highly vetted numeric functions. |
| **Escaping CFFI pointer** | Undefined values or memory crashes | Never return pointers allocated via `with-foreign-object` outside its lexical body. |
| **Dynamic variables across threads** | Shared-state race conditions | Use `let` inside the thread function to establish thread-local variable bindings. |
| **Raw thread storms** | System slowdown due to thread limits | Use `lparallel` worker pools and channels instead of raw `bt:make-thread` in loops. |
| **Generic arithmetic in loops** | SBCL compiler notes show boxing | Provide type declarations (`fixnum`, `double-float`) on loop counters and vectors. |
| **Active threads during image save** | SBCL errors on `save-lisp-and-die` | Terminate all Bordeaux background threads and shut down the `lparallel` kernel before saving. |
| **Ignoring compiler warnings** | Broken system definitions | Turn on compilation checks to treat warnings as errors in CI. |

---

## 8. Code Review Compliance Gate

Before merging Common Lisp code, verify:
1. SBLint or equivalent formatter parses cleanly.
2. The ASDF package compiles with no unhandled warnings or compilation notes.
3. Every CFFI wrapper has been audited for pointer leaks and extent limitations.
4. Multiprocessing routines utilize `lparallel` task kernels.
5. Standing dynamic bindings are documented for thread-local safety.
