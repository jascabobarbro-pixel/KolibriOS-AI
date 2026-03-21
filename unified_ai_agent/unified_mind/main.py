#!/usr/bin/env python3
"""
Unified Mind - Main Entry Point.

This is the main entry point for the Unified Mind AI agent,
providing a command-line interface for natural language interaction
with the KolibriOS AI system.
"""

import asyncio
import argparse
import logging
import os
import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from unified_mind.core import UnifiedMind, UnifiedMindConfig
from unified_mind.interface.cli import CLIInterface

# Configure logging
def setup_logging(level: str = "INFO", log_file: str = None) -> None:
    """Setup logging configuration."""
    handlers = [logging.StreamHandler(sys.stderr)]
    
    if log_file:
        handlers.append(logging.FileHandler(log_file))
    
    logging.basicConfig(
        level=getattr(logging, level.upper(), logging.INFO),
        format="%(asctime)s | %(levelname)-8s | %(name)s | %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        handlers=handlers,
    )


def load_config_from_env() -> UnifiedMindConfig:
    """Load configuration from environment variables."""
    config = UnifiedMindConfig.from_env()
    
    # Override with command-line settings if provided
    if os.environ.get("GEMINI_API_KEY"):
        config.llm.api_key = os.environ.get("GEMINI_API_KEY")
    
    if os.environ.get("LLAMA_MODEL_PATH"):
        config.llm.llama_model_path = os.environ.get("LLAMA_MODEL_PATH")
    
    return config


async def run_interactive(config: UnifiedMindConfig) -> None:
    """Run in interactive CLI mode."""
    mind = UnifiedMind(config)
    
    try:
        await mind.start()
        
        cli = CLIInterface(mind, config=config.interface)
        await cli.start()
        
    except KeyboardInterrupt:
        pass
    finally:
        await mind.stop()


async def run_query(config: UnifiedMindConfig, query: str) -> None:
    """Run a single query and exit."""
    mind = UnifiedMind(config)
    
    try:
        await mind.start()
        
        response = await mind.process(query)
        print(f"\n{response.content}\n")
        
    finally:
        await mind.stop()


async def run_server(config: UnifiedMindConfig, port: int) -> None:
    """Run as a server (web interface placeholder)."""
    mind = UnifiedMind(config)
    
    try:
        await mind.start()
        
        print(f"Unified Mind server running on port {port}")
        print("Press Ctrl+C to stop")
        
        # Keep running
        while True:
            await asyncio.sleep(1)
        
    except KeyboardInterrupt:
        pass
    finally:
        await mind.stop()


def main() -> None:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Unified Mind - KolibriOS AI Agent",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                      # Start interactive mode
  %(prog)s -q "show memory"     # Run single query
  %(prog)s --server --port 8080 # Run as server

Environment Variables:
  GEMINI_API_KEY    API key for Gemini LLM
  LLAMA_MODEL_PATH  Path to local Llama model
  LLM_PROVIDER      LLM provider (gemini, local_llama, auto)
  LLM_MODEL         Model name to use
  LLM_TEMPERATURE   Temperature for generation (0.0-2.0)
  LLM_MAX_TOKENS    Maximum tokens to generate
  MIND_DEBUG        Enable debug mode (true/false)
  MIND_LOG_LEVEL    Log level (DEBUG, INFO, WARNING, ERROR)
        """,
    )
    
    parser.add_argument(
        "-q", "--query",
        type=str,
        help="Run a single query and exit",
    )
    parser.add_argument(
        "--server",
        action="store_true",
        help="Run as a server",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8080,
        help="Port for server mode (default: 8080)",
    )
    parser.add_argument(
        "--log-level",
        type=str,
        default="INFO",
        choices=["DEBUG", "INFO", "WARNING", "ERROR"],
        help="Logging level (default: INFO)",
    )
    parser.add_argument(
        "--log-file",
        type=str,
        help="Log file path",
    )
    parser.add_argument(
        "--config",
        type=str,
        help="Configuration file path (JSON)",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode",
    )
    
    args = parser.parse_args()
    
    # Setup logging
    log_level = "DEBUG" if args.debug else args.log_level
    setup_logging(log_level, args.log_file)
    
    logger = logging.getLogger(__name__)
    
    # Load configuration
    config = load_config_from_env()
    
    if args.debug:
        config.debug = True
        config.log_level = "DEBUG"
    
    # Load configuration file if provided
    if args.config:
        import json
        try:
            with open(args.config, "r") as f:
                config_data = json.load(f)
            # Apply configuration
            for key, value in config_data.items():
                if hasattr(config, key):
                    setattr(config, key, value)
            logger.info(f"Loaded configuration from {args.config}")
        except Exception as e:
            logger.error(f"Could not load config file: {e}")
            sys.exit(1)
    
    # Run appropriate mode
    try:
        if args.query:
            asyncio.run(run_query(config, args.query))
        elif args.server:
            asyncio.run(run_server(config, args.port))
        else:
            asyncio.run(run_interactive(config))
    except KeyboardInterrupt:
        logger.info("Interrupted by user")
    except Exception as e:
        logger.error(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
