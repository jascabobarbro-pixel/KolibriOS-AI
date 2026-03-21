import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'providers/unified_mind_provider.dart';
import 'providers/theme_provider.dart';
import 'providers/file_manager_provider.dart';
import 'providers/creative_assistant_provider.dart';
import 'services/grpc_client.dart';
import 'services/unified_mind_client.dart';
import 'screens/main_dashboard.dart';
import 'theme/app_theme.dart';

/// KolibriOS AI GUI - Adaptive Living Interface
///
/// This is the main entry point for the KolibriOS AI GUI.
/// It provides an adaptive interface that responds to the Unified Mind's directives.
void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize gRPC clients
  final grpcClient = GrpcClient();
  await grpcClient.initialize();
  
  // Initialize Unified Mind client
  final unifiedMindClient = UnifiedMindClient(grpcClient);
  await unifiedMindClient.connect();
  
  runApp(
    ProviderScope(
      child: MultiProvider(
        providers: [
          Provider<GrpcClient>.value(value: grpcClient),
          Provider<UnifiedMindClient>.value(value: unifiedMindClient),
          ChangeNotifierProvider<UnifiedMindProvider>(
            create: (_) => UnifiedMindProvider(unifiedMindClient),
          ),
          ChangeNotifierProvider<ThemeProvider>(
            create: (_) => ThemeProvider(),
          ),
          ChangeNotifierProvider<FileManagerProvider>(
            create: (_) => FileManagerProvider(grpcClient),
          ),
          ChangeNotifierProvider<CreativeAssistantProvider>(
            create: (_) => CreativeAssistantProvider(unifiedMindClient),
          ),
        ],
        child: const KolibriOSApp(),
      ),
    ),
  );
}

/// Main Application Widget
class KolibriOSApp extends ConsumerWidget {
  const KolibriOSApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeProvider).themeMode;
    final appTheme = AppTheme();
    
    return MaterialApp(
      title: 'KolibriOS AI',
      debugShowCheckedModeBanner: false,
      
      // Theme Configuration
      theme: appTheme.lightTheme,
      darkTheme: appTheme.darkTheme,
      themeMode: themeMode,
      
      // Adaptive theming based on Unified Mind
      builder: (context, child) {
        return AnimatedTheme(
          data: Theme.of(context),
          duration: const Duration(milliseconds: 300),
          child: child!,
        );
      },
      
      home: const MainDashboard(),
    );
  }
}
