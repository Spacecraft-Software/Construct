# Spacecraft Dart & Flutter Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Dart and Flutter systems programming. It provides complete, compile-checked configurations and skeletons for Isolate concurrency, controller lifecycle disposal, repaint boundary optimizations, and widget testing.

---

## 1. Concurrency: Offloading CPU Work via `Isolate.run`

Do not block the single-threaded UI event loop. Offload heavy decoding or mathematical tasks to background isolates using `Isolate.run` (Dart 3+).

```dart
import 'dart:convert';
import 'dart:isolate';

class TelemetryRecord {
  final String id;
  final double value;
  final DateTime timestamp;

  const TelemetryRecord({
    required this.id,
    required this.value,
    required this.timestamp,
  });

  factory TelemetryRecord.fromJson(Map<String, dynamic> json) {
    final rawId = json['id'];
    final rawValue = json['value'];
    final rawTime = json['timestamp'];

    if (rawId is! String || rawValue is! num || rawTime is! String) {
      throw const FormatException('Invalid JSON telemetry format');
    }

    return TelemetryRecord(
      id: rawId,
      value: rawValue.toDouble(),
      timestamp: DateTime.parse(rawTime),
    );
  }
}

class TelemetryParser {
  /// Parses a massive JSON payload in the background without blocking the UI thread.
  Future<List<TelemetryRecord>> parseTelemetryBytes(String rawJson) async {
    if (rawJson.length < 50000) {
      // Fall back to main thread execution if payload size is small
      return _decodeSync(rawJson);
    }

    // Isolate.run spawns a worker isolate, runs the closure, and returns the result
    return await Isolate.run(() => _decodeSync(rawJson));
  }

  static List<TelemetryRecord> _decodeSync(String rawJson) {
    final decoded = jsonDecode(rawJson);
    if (decoded is! List) {
      throw const FormatException('Expected JSON list');
    }

    return decoded
        .map((item) {
          if (item is! Map<String, dynamic>) return null;
          try {
            return TelemetryRecord.fromJson(item);
          } catch (_) {
            return null;
          }
        })
        .whereType<TelemetryRecord>()
        .toList();
  }
}
```

---

## 2. StatefulWidget Controller Lifecycle & Disposal

To prevent memory leaks, always close stream controllers and dispose of UI controllers inside the `dispose` method of a `StatefulWidget`.

```dart
import 'package:flutter/material.dart';

class TelemetryFormView extends StatefulWidget {
  const TelemetryFormView({super.key});

  @override
  State<TelemetryFormView> createState() => _TelemetryFormViewState();
}

class _TelemetryFormViewState extends State<TelemetryFormView>
    with SingleTickerProviderStateMixin {
  late final TextEditingController _textController;
  late final AnimationController _animationController;

  @override
  void initState() {
    super.initState();
    // Initialize controller resources
    _textController = TextEditingController();
    _animationController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 2),
    );
  }

  @override
  void dispose() {
    // Critical: Dispose of controllers to prevent memory leaks
    _textController.dispose();
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Column(
        children: [
          TextField(
            controller: _textController,
            decoration: const InputDecoration(labelText: 'Telemetry ID'),
          ),
          RotationTransition(
            turns: _animationController,
            child: const Icon(Icons.sync, size: 50),
          ),
        ],
      ),
    );
  }
}
```

---

## 3. Painting Separation via `RepaintBoundary`

Wrap frequently redrawn visual elements (like spinners or graphs) inside `RepaintBoundary` to prevent global widget tree repaint computations.

```dart
import 'package:flutter/material.dart';

class CustomRadarView extends StatelessWidget {
  const CustomRadarView({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(
      // RepaintBoundary isolates the canvas painting from the static parent UI tree
      child: RepaintBoundary(
        child: AnimatedRadarSpinner(),
      ),
    );
  }
}

class AnimatedRadarSpinner extends StatefulWidget {
  const AnimatedRadarSpinner({super.key});

  @override
  State<AnimatedRadarSpinner> createState() => _AnimatedRadarSpinnerState();
}

class _AnimatedRadarSpinnerState extends State<AnimatedRadarSpinner>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 3),
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return CustomPaint(
          size: const Size(200, 200),
          painter: RadarPainter(angle: _controller.value * 2 * 3.1415),
        );
      },
    );
  }
}

class RadarPainter extends CustomPainter {
  final double angle;

  const RadarPainter({required this.angle});

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.green.withOpacity(0.5)
      ..style = PaintingStyle.fill;
    
    final center = Offset(size.width / 2, size.height / 2);
    canvas.drawCircle(center, size.width / 2, paint);

    // Dynamic radar line drawing
    final linePaint = Paint()
      ..color = Colors.green
      ..strokeWidth = 2.0;
    canvas.drawLine(center, Offset(100 + 100 * 0.5, 100 + 100 * 0.5), linePaint);
  }

  @override
  bool shouldRepaint(covariant RadarPainter oldDelegate) {
    return oldDelegate.angle != angle;
  }
}
```

---

## 4. Testing: Widget Tests

Use `flutter_test` to verify layout structures and widget logic interactions.

```dart
// test/telemetry_widget_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:telemetry_app/telemetry_form_view.dart';

void main() {
  testWidgets('Verify Telemetry Form components load cleanly', (WidgetTester tester) async {
    // Build our widget and trigger a frame.
    await tester.pumpWidget(
      const MaterialApp(
        home: TelemetryFormView(),
      ),
    );

    // Verify text field exists
    expect(find.byType(TextField), findsOneWidget);

    // Verify icons exist
    expect(find.byIcon(Icons.sync), findsOneWidget);

    // Type text into field and verify state
    await tester.enterText(find.byType(TextField), 'packet-102');
    expect(find.text('packet-102'), findsOneWidget);
  });
}
```

---

## 5. Static Analysis Configuration (`analysis_options.yaml`)

Configure warnings as errors and enable performance lints.

```yaml
# analysis_options.yaml
include: package:flutter_lints/flutter.yaml

analyzer:
  language:
    strict-casts: true
    strict-inference: true
    strict-raw-types: true
  errors:
    prefer_const_constructors: error
    close_sinks: error
    avoid_print: warning
    always_specify_types: ignore

linter:
  rules:
    - prefer_const_constructors
    - prefer_const_literals_to_create_immutables
    - prefer_const_constructors_in_immutables
    - cancel_subscriptions
    - close_sinks
    - avoid_unnecessary_containers
    - use_key_in_widget_constructors
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Using `!` force-unwraps** | Runtime crash with Null Check Operator error | Check using optional binding (`??`, `?.`, `is`). |
| **Forgetting to dispose controllers** | Memory leaks, slower performance | Call `.dispose()` in the widget's `dispose()` lifecycle method. |
| **Heavy computations on UI thread** | Frame drops, stuttering animations | Offload computational logic to `Isolate.run()`. |
| **Missing `const` keywords** | Frequent rebuilds, sluggish rendering | Configure `prefer_const_constructors` to throw errors in compiler. |
| **Redrawing complex elements** | Slow layouts on canvas widgets | Wrap canvas painters inside `RepaintBoundary` boxes. |

---

## 7. Code Review Compliance Gate

Before merging Dart/Flutter code, verify:
1. Native Android integrations have been aligned with `@android-skills`.
2. Sound Null Safety is strictly maintained; no `!` operator exists in production code.
3. Every StatefulWidget disposing checklist executes cleanly (controllers and streams closed).
4. Long computations are parsed using `Isolate.run()` worker loops.
5. All constant widgets are flagged with the `const` keyword.
6. Custom canvas paint subtrees are isolated using `RepaintBoundary`.
7. `analysis_options.yaml` has strict-casts enabled and compiles without errors.
