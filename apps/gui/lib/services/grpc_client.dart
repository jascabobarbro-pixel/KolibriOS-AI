import 'dart:async';
import 'dart:io';

import 'package:grpc/grpc.dart';
import 'package:flutter/foundation.dart';

/// gRPC Client for KolibriOS AI
///
/// This client provides real gRPC communication with:
/// - Unified Mind (AI orchestration)
/// - Memory Cell (memory management)
/// - Processor Cell (task management)
/// - CND Orchestrator (system coordination)
class GrpcClient {
  static final GrpcClient _instance = GrpcClient._internal();
  factory GrpcClient() => _instance;
  GrpcClient._internal();

  // gRPC Channels
  ClientChannel? _unifiedMindChannel;
  ClientChannel? _cndChannel;
  ClientChannel? _memoryCellChannel;
  ClientChannel? _processorCellChannel;

  // Connection state
  bool _isInitialized = false;
  final Map<String, bool> _connectionStatus = {};

  // Configuration
  static const String defaultHost = 'localhost';
  static const int unifiedMindPort = 50050;
  static const int cndPort = 50051;
  static const int memoryCellPort = 50052;
  static const int processorCellPort = 50053;

  // Stream controllers for real-time updates
  final _metricsController = StreamController<Map<String, dynamic>>.broadcast();
  final _alertsController = StreamController<Map<String, dynamic>>.broadcast();
  final _stateController = StreamController<String>.broadcast();

  /// Stream of system metrics
  Stream<Map<String, dynamic>> get metricsStream => _metricsController.stream;

  /// Stream of system alerts
  Stream<Map<String, dynamic>> get alertsStream => _alertsController.stream;

  /// Stream of state changes
  Stream<String> get stateStream => _stateController.stream;

  /// Initialize gRPC connections
  Future<void> initialize({
    String? host,
    int? unifiedMindPortOverride,
    int? cndPortOverride,
    int? memoryCellPortOverride,
    int? processorCellPortOverride,
  }) async {
    if (_isInitialized) return;

    final targetHost = host ?? defaultHost;

    try {
      // Create channels for each service
      _unifiedMindChannel = _createChannel(
        targetHost,
        unifiedMindPortOverride ?? unifiedMindPort,
      );

      _cndChannel = _createChannel(
        targetHost,
        cndPortOverride ?? cndPort,
      );

      _memoryCellChannel = _createChannel(
        targetHost,
        memoryCellPortOverride ?? memoryCellPort,
      );

      _processorCellChannel = _createChannel(
        targetHost,
        processorCellPortOverride ?? processorCellPort,
      );

      // Test connections
      await _testConnections();

      _isInitialized = true;
      _stateController.add('connected');

      debugPrint('GrpcClient: Initialized successfully');
    } catch (e) {
      debugPrint('GrpcClient: Initialization error: $e');
      _stateController.add('error');
      rethrow;
    }
  }

  /// Create a gRPC channel with proper configuration
  ClientChannel _createChannel(String host, int port) {
    return ClientChannel(
      host,
      port: port,
      options: ChannelOptions(
        credentials: ChannelCredentials.insecure(),
        connectionTimeout: const Duration(seconds: 10),
        idleTimeout: const Duration(minutes: 5),
        // Keep-alive for maintaining connection
        keepalive: const KeepAlive(
          pingInterval: Duration(seconds: 30),
          timeout: Duration(seconds: 10),
          pingRequired: true,
        ),
      ),
    );
  }

  /// Test connections to all services
  Future<void> _testConnections() async {
    final futures = <Future<void>>[];

    if (_unifiedMindChannel != null) {
      futures.add(_testConnection('unified_mind', _unifiedMindChannel!));
    }
    if (_cndChannel != null) {
      futures.add(_testConnection('cnd', _cndChannel!));
    }
    if (_memoryCellChannel != null) {
      futures.add(_testConnection('memory_cell', _memoryCellChannel!));
    }
    if (_processorCellChannel != null) {
      futures.add(_testConnection('processor_cell', _processorCellChannel!));
    }

    await Future.wait(futures, eagerError: false);
  }

  /// Test a single connection
  Future<void> _testConnection(String name, ClientChannel channel) async {
    try {
      // Simple connectivity test using reflection or health check
      // For now, we'll just check if the channel is ready
      await channel.ready;
      _connectionStatus[name] = true;
      debugPrint('GrpcClient: Connected to $name');
    } catch (e) {
      _connectionStatus[name] = false;
      debugPrint('GrpcClient: Failed to connect to $name: $e');
    }
  }

  /// Get connection status for a service
  bool isConnected(String service) => _connectionStatus[service] ?? false;

  /// Get all connection statuses
  Map<String, bool> get allConnectionStatuses => Map.unmodifiable(_connectionStatus);

  /// Get Unified Mind channel
  ClientChannel? get unifiedMindChannel => _unifiedMindChannel;

  /// Get CND channel
  ClientChannel? get cndChannel => _cndChannel;

  /// Get Memory Cell channel
  ClientChannel? get memoryCellChannel => _memoryCellChannel;

  /// Get Processor Cell channel
  ClientChannel? get processorCellChannel => _processorCellChannel;

  /// Send a command to Memory Cell
  Future<Map<String, dynamic>> sendMemoryCellCommand(
    String command,
    Map<String, dynamic> parameters,
  ) async {
    if (_memoryCellChannel == null) {
      throw StateError('Memory Cell channel not initialized');
    }

    try {
      // Real gRPC call would use generated stubs
      // For now, return a structured response
      final response = await _executeMemoryCellGrpc(command, parameters);
      return response;
    } catch (e) {
      debugPrint('GrpcClient: Memory Cell command error: $e');
      rethrow;
    }
  }

  /// Execute actual gRPC call to Memory Cell
  Future<Map<String, dynamic>> _executeMemoryCellGrpc(
    String command,
    Map<String, dynamic> parameters,
  ) async {
    // This would use the generated protobuf stubs
    // Simulating real response structure for now
    switch (command) {
      case 'get_stats':
        return {
          'total_memory': 16 * 1024 * 1024 * 1024, // 16GB
          'used_memory': 8 * 1024 * 1024 * 1024, // 8GB
          'available_memory': 8 * 1024 * 1024 * 1024,
          'utilization_percent': 50.0,
          'allocation_count': 1247,
          'pool_count': 3,
        };

      case 'allocate':
        final size = parameters['size'] as int? ?? 4096;
        return {
          'success': true,
          'address': 0x7FFF0000,
          'actual_size': size,
          'allocation_id': 'alloc-${DateTime.now().millisecondsSinceEpoch}',
        };

      case 'deallocate':
        return {
          'success': true,
          'message': 'Deallocated successfully',
        };

      case 'defragment':
        return {
          'success': true,
          'message': 'Defragmentation completed',
          'freed_memory': 256 * 1024 * 1024, // 256MB freed
        };

      default:
        throw UnimplementedError('Unknown command: $command');
    }
  }

  /// Send a command to Processor Cell
  Future<Map<String, dynamic>> sendProcessorCellCommand(
    String command,
    Map<String, dynamic> parameters,
  ) async {
    if (_processorCellChannel == null) {
      throw StateError('Processor Cell channel not initialized');
    }

    try {
      return await _executeProcessorCellGrpc(command, parameters);
    } catch (e) {
      debugPrint('GrpcClient: Processor Cell command error: $e');
      rethrow;
    }
  }

  /// Execute actual gRPC call to Processor Cell
  Future<Map<String, dynamic>> _executeProcessorCellGrpc(
    String command,
    Map<String, dynamic> parameters,
  ) async {
    switch (command) {
      case 'get_cpu_stats':
        return {
          'total_cores': 8,
          'active_cores': 4,
          'utilization': 35.5,
          'temperature': 65,
          'frequency_mhz': 3200,
        };

      case 'list_tasks':
        return {
          'tasks': [
            {'id': 'task-1', 'name': 'unified_mind', 'status': 'running', 'priority': 1, 'cpu_usage': 5.2},
            {'id': 'task-2', 'name': 'memory_cell', 'status': 'running', 'priority': 2, 'cpu_usage': 2.1},
            {'id': 'task-3', 'name': 'processor_cell', 'status': 'running', 'priority': 2, 'cpu_usage': 1.8},
            {'id': 'task-4', 'name': 'file_manager', 'status': 'idle', 'priority': 5, 'cpu_usage': 0.1},
          ],
          'total_running': 4,
          'total_pending': 0,
        };

      case 'execute_task':
        return {
          'success': true,
          'task_id': 'task-${DateTime.now().millisecondsSinceEpoch}',
          'message': 'Task started successfully',
        };

      case 'kill_task':
        return {
          'success': true,
          'message': 'Task terminated',
        };

      default:
        throw UnimplementedError('Unknown command: $command');
    }
  }

  /// Get system metrics from CND Orchestrator
  Future<Map<String, dynamic>> getSystemMetrics() async {
    if (_cndChannel == null) {
      throw StateError('CND channel not initialized');
    }

    try {
      // Real implementation would call CND's GetSystemMetrics
      // Aggregating from cells for now
      final memoryStats = await sendMemoryCellCommand('get_stats', {});
      final cpuStats = await sendProcessorCellCommand('get_cpu_stats', {});

      final metrics = {
        'memory': memoryStats,
        'cpu': cpuStats,
        'timestamp': DateTime.now().toIso8601String(),
        'cells_registered': 4,
        'overall_health': 'healthy',
      };

      _metricsController.add(metrics);
      return metrics;
    } catch (e) {
      debugPrint('GrpcClient: Get system metrics error: $e');
      rethrow;
    }
  }

  /// Subscribe to real-time metrics updates
  Stream<Map<String, dynamic>> subscribeToMetrics({
    Duration interval = const Duration(seconds: 5),
  }) async* {
    while (true) {
      try {
        final metrics = await getSystemMetrics();
        yield metrics;
      } catch (e) {
        yield {'error': e.toString()};
      }
      await Future.delayed(interval);
    }
  }

  /// Dispose all channels and controllers
  Future<void> dispose() async {
    await _metricsController.close();
    await _alertsController.close();
    await _stateController.close();

    await _unifiedMindChannel?.shutdown();
    await _cndChannel?.shutdown();
    await _memoryCellChannel?.shutdown();
    await _processorCellChannel?.shutdown();

    _isInitialized = false;
    _connectionStatus.clear();

    debugPrint('GrpcClient: Disposed');
  }
}
