# Spacecraft Flutter & Dart Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Flutter and Dart systems programming. It provides complete, compile-checked configurations and skeletons for Isolate concurrency, controller lifecycle disposal, repaint boundary optimizations, Standard §18 accessibility, and widget testing.

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

## 6. Accessibility: `Semantics`, Announcements & Custom Paint (Standard §18)

§18 makes an accessible mode mandatory for every Spacecraft Software application other than a registered game, and §18.3 names `Semantics` / `SemanticsRole` as Flutter's required bridge. `spacecraft-accessibility-support` owns the §18 contract — the activation toggle, the bridge table, the audit gates; this section covers only the Flutter API surface.

Standard Material widgets carry sensible defaults. Anything icon-only, custom-composed, or custom-painted carries **nothing** until it is declared.

```dart
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

class TelemetryPanel extends StatelessWidget {
  const TelemetryPanel({super.key, required this.packetCount, required this.busy});

  final int packetCount;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    // Platform preferences are read independently of the §18.1 toggle — the
    // user already expressed them system-wide; do not ask twice.
    final reduceMotion = MediaQuery.disableAnimationsOf(context);

    return Column(
      children: [
        // An icon-only button announces as "button" and nothing else without
        // a label. `tooltip` supplies one for free on Material widgets.
        IconButton(
          icon: const Icon(Icons.refresh),
          tooltip: 'Rebuild telemetry index',
          onPressed: busy ? null : () => _reload(context),
        ),

        // Explicit role + label where the widget is composed by hand.
        Semantics(
          label: 'Telemetry packets loaded',
          value: '$packetCount',
          readOnly: true,
          child: Text('$packetCount packets'),
        ),

        // Decoration is excluded so it is skipped rather than read as noise.
        const ExcludeSemantics(child: Divider(height: 1)),

        // §11.0.2 — colour is never the sole carrier of meaning; the status
        // carries a text tag as well as a themed colour.
        Text(
          busy ? '[INFO] Loading…' : '[OK] Idle',
          style: TextStyle(
            color: busy
                ? Theme.of(context).colorScheme.tertiary
                : Theme.of(context).colorScheme.primary,
          ),
        ),

        if (!reduceMotion) const _PulseIndicator(),
      ],
    );
  }

  void _reload(BuildContext context) {
    // State changes that matter are ANNOUNCED, not merely repainted. A spinner
    // that stops communicates nothing to a screen-reader user.
    SemanticsService.announce('Telemetry index rebuilt', TextDirection.ltr);
  }
}
```

**Custom paint is the failure case.** A control drawn onto a canvas has no node in the semantics tree unless the painter publishes one — it is not "partially accessible", it is invisible. `CustomPainter.semanticsBuilder` is how that gap is closed:

```dart
class PacketGraphPainter extends CustomPainter {
  PacketGraphPainter(this.series);

  final List<double> series;

  @override
  void paint(Canvas canvas, Size size) {
    // …painting elided…
  }

  // Without this, the whole graph is one unlabelled region to a screen reader.
  @override
  SemanticsBuilderCallback get semanticsBuilder => (Size size) {
        final width = size.width / series.length;
        return [
          for (var i = 0; i < series.length; i++)
            CustomPainterSemantics(
              rect: Rect.fromLTWH(i * width, 0, width, size.height),
              properties: SemanticsProperties(
                label: 'Sample ${i + 1}',
                value: series[i].toStringAsFixed(2),
                textDirection: TextDirection.ltr,
              ),
            ),
        ];
      };

  @override
  bool shouldRepaint(covariant PacketGraphPainter old) => old.series != series;

  @override
  bool shouldRebuildSemantics(covariant PacketGraphPainter old) => old.series != series;
}
```

**Test it.** Flutter ships accessibility guideline matchers, so the gate is a widget test rather than a manual pass — though §18.4 still requires exercising the built application with a real screen reader (TalkBack, VoiceOver, or Orca) before release.

```dart
testWidgets('telemetry panel meets accessibility guidelines', (tester) async {
  final handle = tester.ensureSemantics();
  await tester.pumpWidget(const MaterialApp(
    home: TelemetryPanel(packetCount: 12, busy: false),
  ));

  await expectLater(tester, meetsGuideline(textContrastGuideline));
  await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
  await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
  await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));

  // The refresh control must be findable BY ITS LABEL, not by icon or position.
  expect(find.bySemanticsLabel('Rebuild telemetry index'), findsOneWidget);

  handle.dispose();
});
```

---

## 7. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Using `!` force-unwraps** | Runtime crash with Null Check Operator error | Check using optional binding (`??`, `?.`, `is`). |
| **Forgetting to dispose controllers** | Memory leaks, slower performance | Call `.dispose()` in the widget's `dispose()` lifecycle method. |
| **Heavy computations on UI thread** | Frame drops, stuttering animations | Offload computational logic to `Isolate.run()`. |
| **Missing `const` keywords** | Frequent rebuilds, sluggish rendering | Configure `prefer_const_constructors` to throw errors in compiler. |
| **Redrawing complex elements** | Slow layouts on canvas widgets | Wrap canvas painters inside `RepaintBoundary` boxes. |
| **Icon-only button, no label** | Screen reader announces "button" and nothing else | Give it a `tooltip`, or wrap in `Semantics(label:)`. |
| **`CustomPainter` without `semanticsBuilder`** | Whole canvas is one unlabelled region; controls are invisible to AT | Publish `CustomPainterSemantics` nodes per control. |
| **Decoration inside `Semantics`** | Screen reader reads dividers and ornaments as content | Wrap decoration in `ExcludeSemantics`. |
| **Spinner stops, nothing announced** | Sighted users see completion; AT users get silence | `SemanticsService.announce(…)` on meaningful state change. |
| **Animating unconditionally** | Motion-sensitive users get animation they disabled system-wide | Gate on `MediaQuery.disableAnimationsOf(context)`. |

---

## 8. Code Review Compliance Gate

Before merging Flutter/Dart code, verify:
1. Native Android integrations have been aligned with `@android-skills`.
2. Sound Null Safety is strictly maintained; no `!` operator exists in production code.
3. Every StatefulWidget disposing checklist executes cleanly (controllers and streams closed).
4. Long computations are parsed using `Isolate.run()` worker loops.
5. All constant widgets are flagged with the `const` keyword.
6. Custom canvas paint subtrees are isolated using `RepaintBoundary`.
7. `analysis_options.yaml` has strict-casts enabled and compiles without errors.
8. Every interactive widget carries a semantic label and role; decoration is wrapped in `ExcludeSemantics` (§18.3).
9. `CustomPainter` surfaces publish `CustomPainterSemantics` nodes via `semanticsBuilder`.
10. State changes that matter to the user are announced, not merely repainted.
11. Reduced motion and high contrast are read from `MediaQuery`, independently of the §18.1 toggle.
12. `meetsGuideline` assertions pass, and the build was exercised with a real screen reader (§18.4).
13. All colors resolve through the named `steelbore` theme; no hex literal appears in widget code (§11.1).
