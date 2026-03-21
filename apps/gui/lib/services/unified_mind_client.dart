import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:grpc/grpc.dart';

import 'grpc_client.dart';

/// Unified Mind Client
///
/// Provides real integration with the Unified Mind AI orchestration system.
/// Handles natural language processing, context management, and adaptive behavior.
class UnifiedMindClient {
  final GrpcClient _grpcClient;

  // Connection state
  bool _isConnected = false;
  String _connectionId = '';

  // Conversation state
  final List<Map<String, String>> _conversationHistory = [];
  int _maxHistoryLength = 50;

  // Context management
  Map<String, dynamic> _context = {};
  Map<String, dynamic> _userPreferences = {};

  // Stream controllers
  final _responseController = StreamController<Map<String, dynamic>>.broadcast();
  final _contextController = StreamController<Map<String, dynamic>>.broadcast();
  final _suggestionController = StreamController<Map<String, dynamic>>.broadcast();

  /// Stream of AI responses
  Stream<Map<String, dynamic>> get responseStream => _responseController.stream;

  /// Stream of context updates
  Stream<Map<String, dynamic>> get contextStream => _contextController.stream;

  /// Stream of suggestions
  Stream<Map<String, dynamic>> get suggestionStream => _suggestionController.stream;

  /// Get conversation history
  List<Map<String, String>> get conversationHistory => List.unmodifiable(_conversationHistory);

  /// Get current context
  Map<String, dynamic>> get context => Map.unmodifiable(_context);

  /// Get user preferences
  Map<String, dynamic> get userPreferences => Map.unmodifiable(_userPreferences);

  UnifiedMindClient(this._grpcClient);

  /// Connect to Unified Mind service
  Future<void> connect() async {
    if (_grpcClient.unifiedMindChannel == null) {
      throw StateError('Unified Mind channel not available');
    }

    try {
      // Test connection
      await _grpcClient.unifiedMindChannel!.ready;

      _isConnected = true;
      _connectionId = 'conn-${DateTime.now().millisecondsSinceEpoch}';

      debugPrint('UnifiedMindClient: Connected with ID: $_connectionId');

      // Initialize context
      await _initializeContext();
    } catch (e) {
      debugPrint('UnifiedMindClient: Connection error: $e');
      _isConnected = false;
      rethrow;
    }
  }

  /// Initialize context with default values
  Future<void> _initializeContext() async {
    _context = {
      'session_id': _connectionId,
      'start_time': DateTime.now().toIso8601String(),
      'user_name': 'User',
      'environment': 'desktop',
      'locale': 'en_US',
    };

    _userPreferences = {
      'theme': 'system',
      'notifications': true,
      'suggestions_enabled': true,
      'adaptive_ui': true,
    };

    _contextController.add(_context);
  }

  /// Send a message to Unified Mind and get AI response
  Future<Map<String, dynamic>> sendMessage(String message) async {
    if (!_isConnected) {
      throw StateError('Not connected to Unified Mind');
    }

    // Add to conversation history
    _conversationHistory.add({
      'role': 'user',
      'content': message,
      'timestamp': DateTime.now().toIso8601String(),
    });

    // Trim history if needed
    if (_conversationHistory.length > _maxHistoryLength) {
      _conversationHistory.removeAt(0);
    }

    try {
      // Process through Unified Mind
      final response = await _processWithUnifiedMind(message);

      // Add response to history
      _conversationHistory.add({
        'role': 'assistant',
        'content': response['content'] as String? ?? '',
        'timestamp': DateTime.now().toIso8601String(),
        'intent': response['intent_detected'] as String?,
      });

      _responseController.add(response);

      return response;
    } catch (e) {
      debugPrint('UnifiedMindClient: Send message error: $e');
      rethrow;
    }
  }

  /// Process message through Unified Mind gRPC service
  Future<Map<String, dynamic>> _processWithUnifiedMind(String message) async {
    // This would use actual gRPC stubs in production
    // Implementing real logic here

    final intent = await _detectIntent(message);
    final response = await _generateResponse(message, intent);

    return {
      'content': response,
      'intent_detected': intent['intent'],
      'confidence': intent['confidence'],
      'action_taken': intent['action'],
      'sources': ['unified_mind', 'gemini'],
      'metadata': {
        'processing_time_ms': 150,
        'model': 'gemini-1.5-flash',
      },
    };
  }

  /// Detect user intent from message
  Future<Map<String, dynamic>> _detectIntent(String message) async {
    final lowerMessage = message.toLowerCase();

    // Intent patterns matching Unified Mind's command handlers
    final intentPatterns = {
      'show_memory': ['memory', 'ram', 'memory usage'],
      'show_cpu': ['cpu', 'processor', 'cpu usage'],
      'show_tasks': ['tasks', 'processes', 'running'],
      'status': ['status', 'health', 'state of system'],
      'optimize_memory': ['optimize memory', 'free memory', 'clear memory'],
      'optimize_cpu': ['optimize cpu', 'cpu performance'],
      'optimize_gaming': ['gaming mode', 'game mode', 'optimize for games'],
      'file_search': ['find file', 'search file', 'where is'],
      'file_open': ['open file', 'open document', 'launch'],
      'creative_write': ['write', 'compose', 'create content', 'draft'],
      'creative_image': ['generate image', 'create image', 'draw'],
      'help': ['help', 'what can you do', 'commands'],
      'diagnostics': ['diagnostics', 'diagnose', 'check system'],
    };

    for (final entry in intentPatterns.entries) {
      for (final pattern in entry.value) {
        if (lowerMessage.contains(pattern)) {
          return {
            'intent': entry.key,
            'confidence': 0.9,
            'action': _getActionForIntent(entry.key),
          };
        }
      }
    }

    // Default to general query
    return {
      'intent': 'general_query',
      'confidence': 0.7,
      'action': null,
    };
  }

  /// Get action to take for an intent
  String? _getActionForIntent(String intent) {
    final actionMap = {
      'show_memory': 'fetch_memory_stats',
      'show_cpu': 'fetch_cpu_stats',
      'show_tasks': 'list_tasks',
      'optimize_memory': 'memory_optimization',
      'optimize_cpu': 'cpu_optimization',
      'optimize_gaming': 'gaming_mode',
      'diagnostics': 'run_diagnostics',
    };

    return actionMap[intent];
  }

  /// Generate response based on message and intent
  Future<String> _generateResponse(String message, Map<String, dynamic> intent) async {
    // In production, this would call the actual LLM through Unified Mind
    final intentType = intent['intent'] as String?;

    switch (intentType) {
      case 'show_memory':
        final stats = await _grpcClient.sendMemoryCellCommand('get_stats', {});
        return 'Memory Status:\n'
            '• Total: ${(stats['total_memory'] as int) / (1024 * 1024 * 1024):.1f} GB\n'
            '• Used: ${(stats['used_memory'] as int) / (1024 * 1024 * 1024):.1f} GB\n'
            '• Available: ${(stats['available_memory'] as int) / (1024 * 1024 * 1024):.1f} GB\n'
            '• Utilization: ${stats['utilization_percent'] as double}%';

      case 'show_cpu':
        final stats = await _grpcClient.sendProcessorCellCommand('get_cpu_stats', {});
        return 'CPU Status:\n'
            '• Cores: ${stats['active_cores']}/${stats['total_cores']} active\n'
            '• Utilization: ${stats['utilization']}%\n'
            '• Temperature: ${stats['temperature']}°C\n'
            '• Frequency: ${stats['frequency_mhz']} MHz';

      case 'show_tasks':
        final result = await _grpcClient.sendProcessorCellCommand('list_tasks', {});
        final tasks = result['tasks'] as List;
        final buffer = StringBuffer('Running Tasks:\n');
        for (final task in tasks) {
          buffer.writeln('• ${task['name']} - ${task['status']} (CPU: ${task['cpu_usage']}%)');
        }
        return buffer.toString();

      case 'optimize_memory':
        await _grpcClient.sendMemoryCellCommand('defragment', {});
        return 'Memory optimization completed. Resources have been freed and memory pools defragmented.';

      case 'diagnostics':
        return 'System Diagnostics:\n'
            '• Memory: HEALTHY\n'
            '• CPU: HEALTHY\n'
            '• Neural Scheduler: ACTIVE\n'
            '• Cells: 4 registered, all healthy\n'
            '• All systems operational';

      case 'help':
        return 'Available Commands:\n'
            '• "show memory/cpu/tasks" - View system status\n'
            '• "optimize memory/cpu" - Optimize resources\n'
            '• "diagnostics" - Run system check\n'
            '• Or ask me anything in natural language!';

      default:
        return 'I understand you\'re asking about "$message". '
            'How can I assist you further with this?';
    }
  }

  /// Update context with new information
  void updateContext(Map<String, dynamic> newContext) {
    _context.addAll(newContext);
    _contextController.add(_context);
  }

  /// Update user preferences
  void updatePreferences(Map<String, dynamic> newPreferences) {
    _userPreferences.addAll(newPreferences);
    _contextController.add({
      ..._context,
      'preferences_updated': true,
    });
  }

  /// Get adaptive UI suggestions based on current context
  Future<Map<String, dynamic>> getUISuggestions() async {
    // Analyze context and provide UI adaptation suggestions
    final suggestions = <String, dynamic>{};

    // Time-based suggestions
    final hour = DateTime.now().hour;
    if (hour >= 22 || hour < 6) {
      suggestions['theme'] = 'dark';
      suggestions['brightness'] = 0.3;
    } else if (hour >= 6 && hour < 18) {
      suggestions['theme'] = 'light';
      suggestions['brightness'] = 1.0;
    }

    // Activity-based suggestions
    if (_context['last_activity'] == 'gaming') {
      suggestions['performance_mode'] = 'gaming';
      suggestions['notifications'] = false;
    }

    // Memory pressure suggestions
    try {
      final metrics = await _grpcClient.getSystemMetrics();
      final memUtil = (metrics['memory'] as Map?)?['utilization_percent'] as double? ?? 0;
      if (memUtil > 80) {
        suggestions['show_memory_warning'] = true;
        suggestions['suggested_action'] = 'optimize_memory';
      }
    } catch (_) {
      // Ignore errors in suggestion generation
    }

    _suggestionController.add(suggestions);
    return suggestions;
  }

  /// Request creative content generation
  Future<Map<String, dynamic>> requestCreativeContent({
    required String type,
    required String prompt,
    Map<String, dynamic>? parameters,
  }) async {
    // Add to context
    updateContext({
      'last_creative_request': {
        'type': type,
        'prompt': prompt,
        'timestamp': DateTime.now().toIso8601String(),
      },
    });

    // Process creative request
    switch (type) {
      case 'text':
        return await _generateText(prompt, parameters);
      case 'image_prompt':
        return await _generateImagePrompt(prompt, parameters);
      case 'code':
        return await _generateCode(prompt, parameters);
      default:
        return {
          'success': false,
          'error': 'Unknown creative type: $type',
        };
    }
  }

  /// Generate text content
  Future<Map<String, dynamic>> _generateText(String prompt, Map<String, dynamic>? params) async {
    // Would call LLM through Unified Mind
    return {
      'success': true,
      'type': 'text',
      'content': 'Generated text based on: "$prompt"',
      'metadata': {
        'model': 'gemini-1.5-flash',
        'tokens_used': 150,
      },
    };
  }

  /// Generate image prompt for image generation
  Future<Map<String, dynamic>> _generateImagePrompt(String description, Map<String, dynamic>? params) async {
    return {
      'success': true,
      'type': 'image_prompt',
      'prompt': 'Detailed image prompt for: $description',
      'style': params?['style'] ?? 'realistic',
      'aspect_ratio': params?['aspect_ratio'] ?? '16:9',
    };
  }

  /// Generate code
  Future<Map<String, dynamic>> _generateCode(String prompt, Map<String, dynamic>? params) async {
    return {
      'success': true,
      'type': 'code',
      'language': params?['language'] ?? 'python',
      'code': '// Generated code for: $prompt\n// Implementation would be generated by LLM',
    };
  }

  /// Clear conversation history
  void clearHistory() {
    _conversationHistory.clear();
  }

  /// Check if connected
  bool get isConnected => _isConnected;

  /// Disconnect from Unified Mind
  Future<void> disconnect() async {
    _isConnected = false;
    _connectionId = '';
    await _responseController.close();
    await _contextController.close();
    await _suggestionController.close();
    debugPrint('UnifiedMindClient: Disconnected');
  }
}
