import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/unified_mind_client.dart';

/// Unified Mind Provider
///
/// Manages the state of the Unified Mind connection and AI responses.
/// Provides real-time updates to the UI based on AI directives.
class UnifiedMindProvider extends ChangeNotifier {
  final UnifiedMindClient _client;

  // State
  bool _isConnected = false;
  bool _isProcessing = false;
  String _lastResponse = '';
  String _currentIntent = '';
  Map<String, dynamic> _context = {};
  Map<String, dynamic> _suggestions = {};
  List<Map<String, String>> _conversationHistory = [];

  // Stream subscriptions
  StreamSubscription? _responseSubscription;
  StreamSubscription? _contextSubscription;
  StreamSubscription? _suggestionSubscription;

  // Error handling
  String? _lastError;

  UnifiedMindProvider(this._client) {
    _initialize();
  }

  /// Initialize provider and subscribe to streams
  void _initialize() {
    _isConnected = _client.isConnected;
    _context = _client.context;
    _conversationHistory = List.from(_client.conversationHistory);

    // Subscribe to response stream
    _responseSubscription = _client.responseStream.listen(
      (response) {
        _lastResponse = response['content'] as String? ?? '';
        _currentIntent = response['intent_detected'] as String? ?? '';
        _isProcessing = false;
        notifyListeners();
      },
      onError: (error) {
        _lastError = error.toString();
        _isProcessing = false;
        notifyListeners();
      },
    );

    // Subscribe to context stream
    _contextSubscription = _client.contextStream.listen(
      (context) {
        _context = context;
        notifyListeners();
      },
    );

    // Subscribe to suggestions stream
    _suggestionSubscription = _client.suggestionStream.listen(
      (suggestions) {
        _suggestions = suggestions;
        notifyListeners();
      },
    );

    // Periodically fetch UI suggestions
    _startSuggestionLoop();
  }

  /// Start periodic suggestion fetching
  void _startSuggestionLoop() {
    Timer.periodic(const Duration(seconds: 30), (_) async {
      if (_isConnected) {
        try {
          await _client.getUISuggestions();
        } catch (e) {
          debugPrint('Suggestion fetch error: $e');
        }
      }
    });
  }

  /// Getters
  bool get isConnected => _isConnected;
  bool get isProcessing => _isProcessing;
  String get lastResponse => _lastResponse;
  String get currentIntent => _currentIntent;
  Map<String, dynamic> get context => Map.unmodifiable(_context);
  Map<String, dynamic> get suggestions => Map.unmodifiable(_suggestions);
  List<Map<String, String>> get conversationHistory => List.unmodifiable(_conversationHistory);
  String? get lastError => _lastError;

  /// Send a message to Unified Mind
  Future<void> sendMessage(String message) async {
    if (!_isConnected) {
      _lastError = 'Not connected to Unified Mind';
      notifyListeners();
      return;
    }

    _isProcessing = true;
    _lastError = null;
    notifyListeners();

    try {
      final response = await _client.sendMessage(message);
      _conversationHistory = List.from(_client.conversationHistory);

      // Execute any action associated with the intent
      if (response['action_taken'] != null) {
        await _executeAction(response['action_taken'] as String);
      }
    } catch (e) {
      _lastError = e.toString();
    } finally {
      _isProcessing = false;
      notifyListeners();
    }
  }

  /// Execute an action based on intent
  Future<void> _executeAction(String action) async {
    debugPrint('Executing action: $action');
  }

  /// Update user preferences
  Future<void> updatePreferences(Map<String, dynamic> preferences) async {
    _client.updatePreferences(preferences);
    notifyListeners();
  }

  /// Update context
  void updateContext(Map<String, dynamic> newContext) {
    _client.updateContext(newContext);
  }

  /// Clear conversation history
  void clearHistory() {
    _client.clearHistory();
    _conversationHistory = [];
    notifyListeners();
  }

  /// Clear error
  void clearError() {
    _lastError = null;
    notifyListeners();
  }

  @override
  void dispose() {
    _responseSubscription?.cancel();
    _contextSubscription?.cancel();
    _suggestionSubscription?.cancel();
    super.dispose();
  }
}

/// Riverpod provider for Unified Mind
final unifiedMindProvider = ChangeNotifierProvider<UnifiedMindProvider>((ref) {
  throw UnimplementedError('Provider must be overridden in MultiProvider');
});

/// Theme Provider
///
/// Manages adaptive theming based on Unified Mind suggestions.
class ThemeProvider extends ChangeNotifier {
  ThemeMode _themeMode = ThemeMode.system;
  Color _accentColor = Colors.blue;
  double _brightness = 1.0;
  bool _adaptiveEnabled = true;
  bool _highContrast = false;
  String _fontFamily = 'Roboto';

  ThemeMode get themeMode => _themeMode;
  Color get accentColor => _accentColor;
  double get brightness => _brightness;
  bool get adaptiveEnabled => _adaptiveEnabled;
  bool get highContrast => _highContrast;
  String get fontFamily => _fontFamily;

  /// Apply suggestions from Unified Mind
  void applySuggestions(Map<String, dynamic> suggestions) {
    if (!_adaptiveEnabled) return;

    if (suggestions.containsKey('theme')) {
      final theme = suggestions['theme'] as String;
      _themeMode = theme == 'dark' ? ThemeMode.dark : ThemeMode.light;
    }

    if (suggestions.containsKey('brightness')) {
      _brightness = suggestions['brightness'] as double;
    }

    if (suggestions.containsKey('high_contrast')) {
      _highContrast = suggestions['high_contrast'] as bool;
    }

    notifyListeners();
  }

  /// Set theme mode manually
  void setThemeMode(ThemeMode mode) {
    _themeMode = mode;
    notifyListeners();
  }

  /// Set accent color
  void setAccentColor(Color color) {
    _accentColor = color;
    notifyListeners();
  }

  /// Toggle adaptive mode
  void setAdaptiveEnabled(bool enabled) {
    _adaptiveEnabled = enabled;
    notifyListeners();
  }

  /// Set high contrast mode
  void setHighContrast(bool enabled) {
    _highContrast = enabled;
    notifyListeners();
  }

  /// Set font family
  void setFontFamily(String family) {
    _fontFamily = family;
    notifyListeners();
  }
}

/// Riverpod provider for Theme
final themeProvider = ChangeNotifierProvider<ThemeProvider>((ref) {
  return ThemeProvider();
});
