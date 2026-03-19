"""
CLI Interface for Unified Mind.

Provides a command-line interface for natural language interaction
with the Unified Mind AI agent.
"""

import asyncio
import logging
import readline
import shlex
import sys
from datetime import datetime
from typing import Any, Optional
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)


class InterfaceState(str, Enum):
    """States of the CLI interface."""
    INITIALIZING = "initializing"
    READY = "ready"
    PROCESSING = "processing"
    SHUTDOWN = "shutdown"


@dataclass
class CLIConfig:
    """Configuration for CLI interface."""
    prompt_string: str = "kolibri> "
    welcome_message: str = """
╔══════════════════════════════════════════════════════════════════════════╗
║                                                                          ║
║   ██████╗  ██████╗ ██╗     ██╗     ██████╗ ███████╗██╗   ██╗            ║
║   ██╔══██╗██╔═══██╗██║     ██║    ██╔════╝ ██╔════╝╚██╗ ██╔╝            ║
║   ██║  ██║██║   ██║██║     ██║    ██║  ███╗ █████╗   ╚████╔╝             ║
║   ██║  ██║██║   ██║██║     ██║    ██║   ██║ ██╔═══╝    ╚██╔╝              ║
║   ██║  ██║╚██████╔╝███████╗███████╗╚██████╔╝███████╗   ██║               ║
║   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚══════╝ ╚═════╝ ╚══════╝   ╚═╝               ║
║                                                                          ║
║                    KolibriOS AI - Unified Mind                           ║
║                    Type 'help' for available commands                    ║
╚══════════════════════════════════════════════════════════════════════════╝
"""
    exit_commands: list[str] = None  # type: ignore
    history_file: Optional[str] = None
    max_history: int = 100
    enable_colors: bool = True
    enable_completion: bool = True

    def __post_init__(self):
        if self.exit_commands is None:
            self.exit_commands = ["exit", "quit", "bye", "goodbye"]


class CLIInterface:
    """
    Command-Line Interface for Unified Mind.
    
    Provides a natural language interface for users to interact with
    the Unified Mind AI agent through text commands.
    """

    def __init__(
        self,
        unified_mind: Any,  # UnifiedMind type
        config: Optional[CLIConfig] = None,
    ) -> None:
        """
        Initialize the CLI interface.
        
        Args:
            unified_mind: The Unified Mind instance to interact with
            config: CLI configuration options
        """
        self.mind = unified_mind
        self.config = config or CLIConfig()
        self.state = InterfaceState.INITIALIZING
        self._history: list[str] = []
        self._running = False
        self._completer = None

    async def start(self) -> None:
        """Start the CLI interface."""
        self._running = True
        self.state = InterfaceState.READY
        
        # Setup readline for command history and completion
        if self.config.enable_completion:
            self._setup_readline()
        
        # Print welcome message
        self._print_welcome()
        
        # Start input loop
        await self._input_loop()

    async def stop(self) -> None:
        """Stop the CLI interface."""
        self._running = False
        self.state = InterfaceState.SHUTDOWN
        
        # Save history
        if self.config.history_file:
            self._save_history()
        
        self._print("\nGoodbye! KolibriOS AI shutting down...")

    def _setup_readline(self) -> None:
        """Setup readline for command history and completion."""
        try:
            readline.set_completer(self._completer_func)
            readline.parse_and_bind("tab: complete")
            readline.set_completer_delims(" \t\n")
            
            # Load history
            if self.config.history_file:
                readline.read_history_file(self.config.history_file)
        except Exception as e:
            logger.warning(f"Could not setup readline: {e}")

    def _completer_func(self, text: str, state: int) -> Optional[str]:
        """Command completion function."""
        commands = [
            "show memory",
            "show cpu",
            "show tasks",
            "status",
            "optimize memory",
            "optimize cpu",
            "optimize gaming",
            "help",
            "clear",
            "diagnostics",
            "restart",
            "exit",
        ]
        
        matches = [cmd for cmd in commands if cmd.startswith(text)]
        
        if state < len(matches):
            return matches[state]
        return None

    async def _input_loop(self) -> None:
        """Main input processing loop."""
        while self._running:
            try:
                # Get user input
                user_input = await self._get_input()
                
                if not user_input:
                    continue
                
                # Check for exit commands
                if user_input.lower().strip() in self.config.exit_commands:
                    break
                
                # Add to history
                self._history.append(user_input)
                if len(self._history) > self.config.max_history:
                    self._history.pop(0)
                
                # Process the input
                self.state = InterfaceState.PROCESSING
                response = await self.mind.process(user_input)
                self.state = InterfaceState.READY
                
                # Display response
                self._print_response(response)
                
            except KeyboardInterrupt:
                break
            except EOFError:
                break
            except Exception as e:
                logger.error(f"Input processing error: {e}")
                self._print_error(f"Error: {e}")

    async def _get_input(self) -> Optional[str]:
        """Get user input asynchronously."""
        loop = asyncio.get_event_loop()
        
        def get_input_sync():
            try:
                return input(self.config.prompt_string)
            except EOFError:
                return None
        
        # Run in executor to not block
        return await loop.run_in_executor(None, get_input_sync)

    def _print_welcome(self) -> None:
        """Print welcome message."""
        if self.config.enable_colors:
            # ANSI colors
            cyan = "\033[96m"
            green = "\033[92m"
            yellow = "\033[93m"
            bold = "\033[1m"
            reset = "\033[0m"
            
            welcome = self.config.welcome_message
            welcome = welcome.replace("KolibriOS AI", f"{bold}{green}KolibriOS AI{reset}")
            welcome = welcome.replace("Unified Mind", f"{cyan}Unified Mind{reset}")
            welcome = welcome.replace("'help'", f"'{yellow}help{reset}'")
            
            print(welcome)
        else:
            print(self.config.welcome_message)

    def _print_response(self, response: Any) -> None:
        """Print the AI response."""
        if self.config.enable_colors:
            bold = "\033[1m"
            reset = "\033[0m"
            print(f"\n{bold}Kolibri:{reset} {response.content}\n")
        else:
            print(f"\nKolibri: {response.content}\n")

    def _print_error(self, message: str) -> None:
        """Print an error message."""
        if self.config.enable_colors:
            red = "\033[91m"
            reset = "\033[0m"
            print(f"\n{red}Error: {message}{reset}\n")
        else:
            print(f"\nError: {message}\n")

    def _print(self, message: str) -> None:
        """Print a message."""
        print(message)

    def _save_history(self) -> None:
        """Save command history to file."""
        try:
            readline.write_history_file(self.config.history_file)
        except Exception as e:
            logger.warning(f"Could not save history: {e}")

    async def run_script(self, script_path: str) -> None:
        """
        Run a script file with multiple commands.
        
        Args:
            script_path: Path to script file
        """
        try:
            with open(script_path, "r") as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#"):
                        response = await self.mind.process(line)
                        self._print_response(response)
        except FileNotFoundError:
            self._print_error(f"Script file not found: {script_path}")
        except Exception as e:
            self._print_error(f"Script error: {e}")


async def run_cli(unified_mind: Any) -> None:
    """
    Run the CLI interface.
    
    Args:
        unified_mind: The Unified Mind instance
    """
    cli = CLIInterface(unified_mind)
    
    try:
        await cli.start()
    except KeyboardInterrupt:
        pass
    finally:
        await cli.stop()
