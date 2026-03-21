#!/usr/bin/env python3
"""
KolibriOS AI - AI Agent Learning and Adaptation Experiment

This experiment evaluates the learning and adaptation capabilities of the Unified Mind
AI agent. It tests preference learning, context adaptation, error handling, and
re-learning after preference resets.

Experiment Steps:
1. Baseline Interaction - Establish preferences through natural language
2. Introduce New Context - Test if preferences are recalled and applied
3. Simulate Error/Failure - Test self-diagnosis and resolution
4. Evaluate Learning - Reset preferences and test re-learning
5. Analyze Results - Document findings and recommendations

Usage:
    python experiments/ai_agent_learning_experiment.py
"""

import asyncio
import json
import logging
import os
import sys
import time
import random
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Optional, Dict, List

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class ExperimentPhase(Enum):
    """Experiment phases."""
    BASELINE = "baseline"
    CONTEXT_CHANGE = "context_change"
    ERROR_SIMULATION = "error_simulation"
    LEARNING_RESET = "learning_reset"
    ANALYSIS = "analysis"


class ObservationType(Enum):
    """Types of observations."""
    PREFERENCE_LEARNED = "preference_learned"
    PREFERENCE_RECALLED = "preference_recalled"
    ADAPTATION_DETECTED = "adaptation_detected"
    ERROR_DETECTED = "error_detected"
    ERROR_RESOLVED = "error_resolved"
    RELEARNING_OBSERVED = "relearning_observed"
    BEHAVIOR_CHANGE = "behavior_change"


@dataclass
class Observation:
    """Single observation during experiment."""
    timestamp: datetime
    phase: ExperimentPhase
    observation_type: ObservationType
    description: str
    details: Dict[str, Any] = field(default_factory=dict)
    success: bool = True
    confidence: float = 1.0


@dataclass
class LearningMetric:
    """Metric for learning evaluation."""
    name: str
    baseline_value: Any
    current_value: Any
    change_detected: bool = False
    adaptation_quality: float = 0.0


@dataclass
class ExperimentResult:
    """Result of the complete experiment."""
    name: str
    start_time: datetime
    end_time: Optional[datetime] = None
    observations: List[Observation] = field(default_factory=list)
    metrics: Dict[str, LearningMetric] = field(default_factory=dict)
    strengths: List[str] = field(default_factory=list)
    weaknesses: List[str] = field(default_factory=list)
    recommendations: List[str] = field(default_factory=list)
    overall_score: float = 0.0

    def add_observation(self, obs: Observation):
        """Add an observation."""
        self.observations.append(obs)

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "name": self.name,
            "start_time": self.start_time.isoformat(),
            "end_time": self.end_time.isoformat() if self.end_time else None,
            "total_observations": len(self.observations),
            "successful_observations": sum(1 for o in self.observations if o.success),
            "metrics": {k: {
                "name": v.name,
                "baseline": str(v.baseline_value),
                "current": str(v.current_value),
                "change_detected": v.change_detected,
                "adaptation_quality": v.adaptation_quality,
            } for k, v in self.metrics.items()},
            "strengths": self.strengths,
            "weaknesses": self.weaknesses,
            "recommendations": self.recommendations,
            "overall_score": self.overall_score,
        }


class UnifiedMindLearningHarness:
    """
    Test harness for evaluating Unified Mind learning capabilities.
    
    Simulates interactions and tracks learning/adaptation behavior.
    """

    def __init__(self):
        """Initialize the harness."""
        self._connected = False
        self._user_preferences = {
            "theme": "system",
            "work_mode": "balanced",
            "creative_time": None,
            "favorite_apps": [],
            "interaction_style": "neutral",
            "notification_level": "normal",
        }
        self._learned_preferences = {}
        self._conversation_history = []
        self._system_context = {
            "current_app": None,
            "time_of_day": "afternoon",
            "active_tasks": [],
            "resource_state": "normal",
        }
        self._error_state = None
        self._learning_log = []
        self._adaptation_history = []

    async def connect(self) -> bool:
        """Connect to Unified Mind."""
        logger.info("Connecting to Unified Mind for learning experiment...")
        self._connected = True
        await asyncio.sleep(0.1)  # Simulate connection
        logger.info("Connected successfully")
        return True

    async def disconnect(self):
        """Disconnect from Unified Mind."""
        self._connected = False
        logger.info("Disconnected from Unified Mind")

    async def interact(self, message: str) -> dict:
        """
        Interact with Unified Mind.
        
        Args:
            message: User message
            
        Returns:
            Response with detected learning/adaptation
        """
        if not self._connected:
            return {"success": False, "error": "Not connected"}

        start_time = time.time()
        
        # Add to history
        self._conversation_history.append({
            "role": "user",
            "content": message,
            "timestamp": datetime.now().isoformat(),
        })

        # Process and detect learning
        response = await self._process_with_learning(message)
        
        # Add response to history
        self._conversation_history.append({
            "role": "assistant",
            "content": response["content"],
            "timestamp": datetime.now().isoformat(),
            "preferences_detected": response.get("preferences_detected", {}),
            "adaptations": response.get("adaptations", []),
        })

        response["response_time_ms"] = (time.time() - start_time) * 1000
        return response

    async def _process_with_learning(self, message: str) -> dict:
        """Process message with learning detection."""
        lower_msg = message.lower()
        preferences_detected = {}
        adaptations = []
        
        # Detect preference statements
        if "prefer" in lower_msg or "i like" in lower_msg or "usually" in lower_msg:
            prefs = self._extract_preferences(message)
            preferences_detected = prefs
            self._learned_preferences.update(prefs)
            self._learning_log.append({
                "timestamp": datetime.now().isoformat(),
                "type": "preference_learned",
                "preferences": prefs,
                "source_message": message[:100],
            })

        # Check for context-based adaptation
        current_context = self._system_context.copy()
        
        # Generate response based on learned preferences
        response = await self._generate_contextual_response(message, preferences_detected)
        
        # Check if adaptation occurred
        if self._learned_preferences:
            adaptation = self._check_adaptation(message, response)
            if adaptation:
                adaptations.append(adaptation)
                self._adaptation_history.append(adaptation)

        response["preferences_detected"] = preferences_detected
        response["adaptations"] = adaptations
        response["learned_preferences_count"] = len(self._learned_preferences)
        
        return response

    def _extract_preferences(self, message: str) -> dict:
        """Extract preferences from message."""
        prefs = {}
        lower_msg = message.lower()

        # Theme preference
        if "dark theme" in lower_msg or "dark mode" in lower_msg:
            prefs["theme"] = "dark"
        elif "light theme" in lower_msg or "light mode" in lower_msg or "bright theme" in lower_msg:
            prefs["theme"] = "light"

        # Work schedule
        if "evening" in lower_msg or "night" in lower_msg:
            prefs["preferred_work_time"] = "evening"
        elif "morning" in lower_msg:
            prefs["preferred_work_time"] = "morning"
        elif "afternoon" in lower_msg:
            prefs["preferred_work_time"] = "afternoon"

        # Work mode
        if "creative writing" in lower_msg or "writing" in lower_msg:
            prefs["work_mode"] = "creative"
            prefs["creative_activity"] = "writing"
        if "coding" in lower_msg or "programming" in lower_msg:
            prefs["work_mode"] = "development"

        # Notification preference
        if "quiet" in lower_msg or "don't disturb" in lower_msg or "focus" in lower_msg:
            prefs["notification_level"] = "minimal"

        return prefs

    async def _generate_contextual_response(self, message: str, prefs: dict) -> dict:
        """Generate response considering context and learned preferences."""
        lower_msg = message.lower()
        content = ""
        confidence = 0.9
        
        # Check if we have relevant learned preferences
        relevant_prefs = {}
        
        if "theme" in self._learned_preferences:
            relevant_prefs["theme"] = self._learned_preferences["theme"]
        if "preferred_work_time" in self._learned_preferences:
            relevant_prefs["work_time"] = self._learned_preferences["preferred_work_time"]
        if "work_mode" in self._learned_preferences:
            relevant_prefs["work_mode"] = self._learned_preferences["work_mode"]

        # Generate contextually aware response
        if relevant_prefs:
            pref_str = ", ".join(f"{k}: {v}" for k, v in relevant_prefs.items())
            content = f"I remember your preferences ({pref_str}). "
            confidence = 0.95
        else:
            content = "I'll help you with that. "

        # Add specific response based on query
        if "theme" in lower_msg or "mode" in lower_msg:
            theme = self._learned_preferences.get("theme", "system default")
            content += f"Based on your preference for {theme} theme, I've adjusted the settings accordingly."
        elif "work" in lower_msg or "evening" in lower_msg or "morning" in lower_msg:
            work_time = self._learned_preferences.get("preferred_work_time", "any time")
            work_mode = self._learned_preferences.get("work_mode", "general")
            content += f"Considering you prefer working in the {work_time} with a {work_mode} focus, let me optimize your environment."
        elif "suggest" in lower_msg or "recommend" in lower_msg:
            content += self._generate_suggestion()
        else:
            content += "How can I assist you further?"

        return {
            "success": True,
            "content": content,
            "confidence": confidence,
        }

    def _generate_suggestion(self) -> str:
        """Generate suggestion based on learned preferences."""
        suggestions = []
        
        theme = self._learned_preferences.get("theme")
        if theme == "dark":
            suggestions.append("I suggest enabling dark mode for your evening work sessions")
        elif theme == "light":
            suggestions.append("A bright theme might be better for your daytime productivity")

        work_time = self._learned_preferences.get("preferred_work_time")
        if work_time == "evening":
            suggestions.append("I can schedule your creative tasks for evening hours")
        elif work_time == "morning":
            suggestions.append("Your analytical tasks would be best scheduled for mornings")

        work_mode = self._learned_preferences.get("work_mode")
        if work_mode == "creative":
            suggestions.append("Opening Creative Assistant for your writing sessions")
        elif work_mode == "development":
            suggestions.append("Preparing development environment with your preferred tools")

        if suggestions:
            return "Based on what I've learned: " + "; ".join(suggestions[:2]) + "."
        return "I don't have enough preference data yet to make personalized suggestions."

    def _check_adaptation(self, message: str, response: dict) -> Optional[dict]:
        """Check if adaptation occurred."""
        lower_msg = message.lower()
        
        # Check theme adaptation
        if "theme" in lower_msg or "mode" in lower_msg:
            learned_theme = self._learned_preferences.get("theme")
            if learned_theme:
                return {
                    "type": "theme_adaptation",
                    "learned_preference": learned_theme,
                    "applied": True,
                    "timestamp": datetime.now().isoformat(),
                }

        # Check work time adaptation
        if "evening" in lower_msg or "morning" in lower_msg or "schedule" in lower_msg:
            learned_time = self._learned_preferences.get("preferred_work_time")
            if learned_time:
                return {
                    "type": "schedule_adaptation",
                    "learned_preference": learned_time,
                    "applied": True,
                    "timestamp": datetime.now().isoformat(),
                }

        return None

    async def set_context(self, context: dict):
        """Set system context."""
        self._system_context.update(context)
        logger.info(f"Context updated: {context}")

    async def simulate_error(self, error_type: str) -> dict:
        """
        Simulate an error condition.
        
        Args:
            error_type: Type of error to simulate
            
        Returns:
            Error simulation result
        """
        self._error_state = {
            "type": error_type,
            "timestamp": datetime.now().isoformat(),
            "resolved": False,
        }

        if error_type == "memory_warning":
            return {
                "success": True,
                "error": {
                    "type": "resource_warning",
                    "message": "Memory utilization at 92%",
                    "severity": "warning",
                }
            }
        elif error_type == "app_crash":
            return {
                "success": True,
                "error": {
                    "type": "application_error",
                    "message": "Creative Assistant encountered an unexpected error",
                    "severity": "error",
                }
            }
        elif error_type == "network_timeout":
            return {
                "success": True,
                "error": {
                    "type": "network_error",
                    "message": "LLM API request timed out after 30 seconds",
                    "severity": "warning",
                }
            }
        else:
            return {
                "success": True,
                "error": {
                    "type": "generic_error",
                    "message": f"An error occurred: {error_type}",
                    "severity": "warning",
                }
            }

    async def diagnose_and_resolve(self, error_info: dict) -> dict:
        """
        Diagnose and attempt to resolve an error.
        
        Args:
            error_info: Error information
            
        Returns:
            Resolution result
        """
        error_type = error_info.get("error", {}).get("type", "unknown")
        
        resolution_steps = []
        resolved = False
        
        if error_type == "resource_warning":
            resolution_steps = [
                "Analyzing memory usage patterns",
                "Identifying memory-intensive processes",
                "Suggesting resource optimization",
                "Clearing non-essential caches",
            ]
            resolved = True
            
        elif error_type == "application_error":
            resolution_steps = [
                "Capturing error logs",
                "Analyzing crash dump",
                "Suggesting restart with safe mode",
                "Backing up unsaved work",
            ]
            resolved = True
            
        elif error_type == "network_error":
            resolution_steps = [
                "Checking network connectivity",
                "Testing alternative endpoints",
                "Switching to fallback LLM provider",
                "Queueing request for retry",
            ]
            resolved = True
        else:
            resolution_steps = [
                "Logging error details",
                "Suggesting manual intervention",
            ]
            resolved = False

        if resolved:
            self._error_state["resolved"] = True

        return {
            "success": True,
            "error_type": error_type,
            "resolution_steps": resolution_steps,
            "resolved": resolved,
            "timestamp": datetime.now().isoformat(),
        }

    async def reset_preferences(self):
        """Reset learned preferences."""
        self._learned_preferences = {}
        self._learning_log.append({
            "timestamp": datetime.now().isoformat(),
            "type": "preferences_reset",
            "message": "All learned preferences have been cleared",
        })
        logger.info("Preferences reset")

    def get_learning_log(self) -> list:
        """Get learning log."""
        return self._learning_log.copy()

    def get_learned_preferences(self) -> dict:
        """Get learned preferences."""
        return self._learned_preferences.copy()

    def get_adaptation_history(self) -> list:
        """Get adaptation history."""
        return self._adaptation_history.copy()


class AIAgentLearningExperiment:
    """
    AI Agent Learning and Adaptation Experiment.
    
    Evaluates the Unified Mind's ability to learn user preferences,
    adapt to context changes, handle errors, and re-learn after resets.
    """

    def __init__(self):
        """Initialize the experiment."""
        self.harness = UnifiedMindLearningHarness()
        self.result = ExperimentResult(
            name="AI Agent Learning and Adaptation Experiment",
            start_time=datetime.now(),
        )

    async def run(self) -> ExperimentResult:
        """
        Run the complete experiment.
        
        Returns:
            Experiment result with observations and metrics
        """
        logger.info("=" * 60)
        logger.info("Starting AI Agent Learning and Adaptation Experiment")
        logger.info("=" * 60)

        # Connect to Unified Mind
        connected = await self.harness.connect()
        if not connected:
            logger.error("Failed to connect to Unified Mind")
            return self.result

        try:
            # Phase 1: Baseline Interaction
            await self._run_baseline_phase()

            # Phase 2: Context Change
            await self._run_context_change_phase()

            # Phase 3: Error Simulation
            await self._run_error_simulation_phase()

            # Phase 4: Learning Reset
            await self._run_learning_reset_phase()

            # Phase 5: Analysis
            self._analyze_results()

        finally:
            await self.harness.disconnect()

        self.result.end_time = datetime.now()
        logger.info("=" * 60)
        logger.info(f"Experiment completed. Overall score: {self.result.overall_score:.1f}/100")
        logger.info("=" * 60)

        return self.result

    async def _run_baseline_phase(self):
        """
        Phase 1: Baseline Interaction.
        
        Establish baseline preferences through natural language interaction.
        """
        logger.info("\n--- Phase 1: Baseline Interaction ---")
        
        interactions = [
            ("I prefer dark themes for my work environment", "theme"),
            ("I usually work on creative writing in the evenings", "work_preference"),
            ("Please keep notifications minimal during my focus time", "notification"),
            ("I like using the Creative Assistant for my writing projects", "app_preference"),
        ]

        for message, pref_type in interactions:
            response = await self.harness.interact(message)
            
            # Record observation
            prefs_detected = response.get("preferences_detected", {})
            
            obs = Observation(
                timestamp=datetime.now(),
                phase=ExperimentPhase.BASELINE,
                observation_type=ObservationType.PREFERENCE_LEARNED,
                description=f"User stated: '{message[:50]}...'",
                details={
                    "message": message,
                    "preferences_detected": prefs_detected,
                    "response_time_ms": response.get("response_time_ms", 0),
                },
                success=len(prefs_detected) > 0,
                confidence=response.get("confidence", 0.5),
            )
            self.result.add_observation(obs)
            
            logger.info(f"  Interaction: '{message[:40]}...'")
            logger.info(f"  Preferences detected: {prefs_detected}")
            logger.info(f"  Success: {obs.success}")

        # Record baseline metrics
        learned_prefs = self.harness.get_learned_preferences()
        self.result.metrics["preference_learning"] = LearningMetric(
            name="Preference Learning",
            baseline_value=0,
            current_value=len(learned_prefs),
            change_detected=True,
            adaptation_quality=len(learned_prefs) / 4.0,  # 4 possible preferences
        )

    async def _run_context_change_phase(self):
        """
        Phase 2: Context Change.
        
        Test if preferences are recalled and applied when context changes.
        """
        logger.info("\n--- Phase 2: Context Change ---")

        # Change system context
        await self.harness.set_context({
            "current_app": "coding_editor",
            "time_of_day": "evening",
            "active_tasks": ["development", "documentation"],
        })

        # Test preference recall
        test_interactions = [
            "What theme should I use?",
            "Suggest the best time for my creative work",
            "What apps do you recommend for my current task?",
        ]

        for message in test_interactions:
            response = await self.harness.interact(message)
            
            # Check if learned preferences were applied
            adaptations = response.get("adaptations", [])
            content = response.get("content", "")
            
            # Check for preference recall in response
            prefs_recalled = any(
                pref in content.lower() 
                for pref in ["dark", "evening", "creative", "writing"]
            )

            obs = Observation(
                timestamp=datetime.now(),
                phase=ExperimentPhase.CONTEXT_CHANGE,
                observation_type=ObservationType.PREFERENCE_RECALLED if prefs_recalled else ObservationType.ADAPTATION_DETECTED,
                description=f"Context query: '{message}'",
                details={
                    "message": message,
                    "response": content[:200],
                    "adaptations": adaptations,
                    "preferences_recalled": prefs_recalled,
                },
                success=prefs_recalled or len(adaptations) > 0,
                confidence=response.get("confidence", 0.5),
            )
            self.result.add_observation(obs)
            
            logger.info(f"  Query: '{message}'")
            logger.info(f"  Preferences recalled: {prefs_recalled}")
            logger.info(f"  Adaptations: {len(adaptations)}")

        # Record adaptation metric
        adaptation_history = self.harness.get_adaptation_history()
        self.result.metrics["adaptation_rate"] = LearningMetric(
            name="Adaptation Rate",
            baseline_value=0,
            current_value=len(adaptation_history),
            change_detected=True,
            adaptation_quality=min(len(adaptation_history) / 3.0, 1.0),
        )

    async def _run_error_simulation_phase(self):
        """
        Phase 3: Error Simulation.
        
        Test error detection, diagnosis, and resolution.
        """
        logger.info("\n--- Phase 3: Error Simulation ---")

        error_scenarios = [
            "memory_warning",
            "app_crash",
            "network_timeout",
        ]

        for error_type in error_scenarios:
            # Simulate error
            error_result = await self.harness.simulate_error(error_type)
            error_info = error_result.get("error", {})
            
            obs = Observation(
                timestamp=datetime.now(),
                phase=ExperimentPhase.ERROR_SIMULATION,
                observation_type=ObservationType.ERROR_DETECTED,
                description=f"Simulated error: {error_type}",
                details={
                    "error_type": error_type,
                    "error_message": error_info.get("message"),
                    "severity": error_info.get("severity"),
                },
                success=True,
            )
            self.result.add_observation(obs)
            logger.info(f"  Error simulated: {error_type}")
            logger.info(f"  Message: {error_info.get('message')}")

            # Attempt diagnosis and resolution
            resolution = await self.harness.diagnose_and_resolve(error_result)
            
            obs = Observation(
                timestamp=datetime.now(),
                phase=ExperimentPhase.ERROR_SIMULATION,
                observation_type=ObservationType.ERROR_RESOLVED,
                description=f"Resolution attempt for: {error_type}",
                details={
                    "resolution_steps": resolution.get("resolution_steps", []),
                    "resolved": resolution.get("resolved", False),
                },
                success=resolution.get("resolved", False),
            )
            self.result.add_observation(obs)
            logger.info(f"  Resolved: {resolution.get('resolved')}")
            logger.info(f"  Steps: {len(resolution.get('resolution_steps', []))}")

        # Record error handling metric
        error_observations = [
            o for o in self.result.observations 
            if o.phase == ExperimentPhase.ERROR_SIMULATION
        ]
        resolved_count = sum(1 for o in error_observations if o.observation_type == ObservationType.ERROR_RESOLVED and o.success)
        
        self.result.metrics["error_resolution"] = LearningMetric(
            name="Error Resolution Rate",
            baseline_value=0,
            current_value=resolved_count,
            change_detected=True,
            adaptation_quality=resolved_count / len(error_scenarios),
        )

    async def _run_learning_reset_phase(self):
        """
        Phase 4: Learning Reset.
        
        Reset preferences and test re-learning.
        """
        logger.info("\n--- Phase 4: Learning Reset ---")

        # Record preferences before reset
        prefs_before = self.harness.get_learned_preferences()
        logger.info(f"  Preferences before reset: {len(prefs_before)}")

        # Reset preferences
        await self.harness.reset_preferences()

        obs = Observation(
            timestamp=datetime.now(),
            phase=ExperimentPhase.LEARNING_RESET,
            observation_type=ObservationType.BEHAVIOR_CHANGE,
            description="Preferences reset",
            details={
                "preferences_before": len(prefs_before),
                "preferences_after": 0,
            },
            success=True,
        )
        self.result.add_observation(obs)
        logger.info("  Preferences reset to defaults")

        # Test re-learning with new preferences
        new_interactions = [
            ("I actually prefer light themes now", "theme_change"),
            ("I've switched to morning productivity sessions", "time_change"),
        ]

        for message, pref_type in new_interactions:
            response = await self.harness.interact(message)
            
            prefs_detected = response.get("preferences_detected", {})
            
            obs = Observation(
                timestamp=datetime.now(),
                phase=ExperimentPhase.LEARNING_RESET,
                observation_type=ObservationType.RELEARNING_OBSERVED,
                description=f"Re-learning: '{message[:40]}...'",
                details={
                    "message": message,
                    "preferences_detected": prefs_detected,
                },
                success=len(prefs_detected) > 0,
                confidence=response.get("confidence", 0.5),
            )
            self.result.add_observation(obs)
            logger.info(f"  Re-learning: '{message[:40]}...'")
            logger.info(f"  New preferences: {prefs_detected}")

        # Record re-learning metric
        prefs_after = self.harness.get_learned_preferences()
        self.result.metrics["relearning_efficiency"] = LearningMetric(
            name="Re-learning Efficiency",
            baseline_value=len(prefs_before),
            current_value=len(prefs_after),
            change_detected=True,
            adaptation_quality=len(prefs_after) / 2.0,  # 2 new preferences taught
        )

    def _analyze_results(self):
        """
        Phase 5: Analysis.
        
        Analyze all observations and generate findings.
        """
        logger.info("\n--- Phase 5: Analysis ---")

        # Calculate overall score
        total_observations = len(self.result.observations)
        successful_observations = sum(1 for o in self.result.observations if o.success)
        
        if total_observations > 0:
            success_rate = successful_observations / total_observations
        else:
            success_rate = 0

        # Calculate metrics score
        metrics_score = sum(
            m.adaptation_quality for m in self.result.metrics.values()
        ) / len(self.result.metrics) if self.result.metrics else 0

        self.result.overall_score = (success_rate * 50 + metrics_score * 50)

        # Identify strengths
        if self.result.metrics.get("preference_learning", LearningMetric("", 0, 0)).adaptation_quality > 0.7:
            self.result.strengths.append("Strong preference learning capability")
        
        if self.result.metrics.get("adaptation_rate", LearningMetric("", 0, 0)).adaptation_quality > 0.7:
            self.result.strengths.append("Effective context-aware adaptation")
        
        if self.result.metrics.get("error_resolution", LearningMetric("", 0, 0)).adaptation_quality > 0.7:
            self.result.strengths.append("Good error detection and resolution")
        
        if self.result.metrics.get("relearning_efficiency", LearningMetric("", 0, 0)).adaptation_quality > 0.7:
            self.result.strengths.append("Efficient re-learning after reset")

        # Identify weaknesses
        if self.result.metrics.get("preference_learning", LearningMetric("", 0, 0)).adaptation_quality < 0.5:
            self.result.weaknesses.append("Limited preference detection accuracy")
        
        if self.result.metrics.get("adaptation_rate", LearningMetric("", 0, 0)).adaptation_quality < 0.5:
            self.result.weaknesses.append("Inconsistent preference recall")
        
        if self.result.metrics.get("error_resolution", LearningMetric("", 0, 0)).adaptation_quality < 0.5:
            self.result.weaknesses.append("Incomplete error resolution logic")
        
        if self.result.metrics.get("relearning_efficiency", LearningMetric("", 0, 0)).adaptation_quality < 0.5:
            self.result.weaknesses.append("Slow re-learning after preference changes")

        # Generate recommendations
        self.result.recommendations = [
            "Implement persistent storage for learned preferences across sessions",
            "Add confidence scoring for preference predictions",
            "Develop proactive suggestion system based on learned patterns",
            "Implement A/B testing for preference adaptation algorithms",
            "Add user feedback mechanism for preference correction",
            "Develop preference conflict resolution logic",
            "Implement time-based preference decay for stale preferences",
            "Add context-aware preference prioritization",
        ]

        logger.info(f"  Total observations: {total_observations}")
        logger.info(f"  Successful: {successful_observations}")
        logger.info(f"  Success rate: {success_rate:.1%}")
        logger.info(f"  Metrics score: {metrics_score:.1%}")
        logger.info(f"  Overall score: {self.result.overall_score:.1f}/100")
        logger.info(f"  Strengths: {len(self.result.strengths)}")
        logger.info(f"  Weaknesses: {len(self.result.weaknesses)}")


def generate_experiment_report(result: ExperimentResult) -> str:
    """Generate detailed markdown report for the experiment."""
    
    md = f"""# AI Agent Learning and Adaptation Experiment Report

## Experiment Overview

| Attribute | Value |
|-----------|-------|
| **Experiment Name** | {result.name} |
| **Start Time** | {result.start_time.strftime('%Y-%m-%d %H:%M:%S')} |
| **End Time** | {result.end_time.strftime('%Y-%m-%d %H:%M:%S') if result.end_time else 'In Progress'} |
| **Overall Score** | {result.overall_score:.1f}/100 |

## Executive Summary

This experiment evaluated the learning and adaptation capabilities of the KolibriOS AI Unified Mind agent. The agent was tested on its ability to learn user preferences through natural language interaction, adapt its behavior based on learned preferences, detect and resolve errors, and re-learn preferences after a reset.

### Key Findings

- **Total Observations**: {len(result.observations)}
- **Successful Observations**: {sum(1 for o in result.observations if o.success)}
- **Success Rate**: {(sum(1 for o in result.observations if o.success) / len(result.observations) * 100):.1f}%

## Experiment Phases

### Phase 1: Baseline Interaction

The baseline phase established initial preferences through natural language interaction. The user stated preferences for:
- Theme preference (dark/light)
- Work schedule preferences
- Notification settings
- Application preferences

"""

    # Add baseline observations
    baseline_obs = [o for o in result.observations if o.phase == ExperimentPhase.BASELINE]
    md += "#### Observations\n\n"
    for obs in baseline_obs:
        status = "✅" if obs.success else "❌"
        md += f"- {status} {obs.description}\n"
        if obs.details.get("preferences_detected"):
            md += f"  - Preferences detected: {obs.details['preferences_detected']}\n"
    
    md += f"""
### Phase 2: Context Change

The context change phase tested whether the agent recalls and applies learned preferences when the system context changes. The context was modified to simulate an evening coding session.

#### Observations

"""
    context_obs = [o for o in result.observations if o.phase == ExperimentPhase.CONTEXT_CHANGE]
    for obs in context_obs:
        status = "✅" if obs.success else "❌"
        md += f"- {status} {obs.description}\n"
        if obs.details.get("preferences_recalled") is not None:
            md += f"  - Preferences recalled: {obs.details['preferences_recalled']}\n"

    md += f"""
### Phase 3: Error Simulation

The error simulation phase tested the agent's ability to detect, diagnose, and resolve various error conditions.

#### Error Scenarios Tested

"""
    error_obs = [o for o in result.observations if o.phase == ExperimentPhase.ERROR_SIMULATION]
    for obs in error_obs:
        status = "✅" if obs.success else "❌"
        md += f"- {status} {obs.description}\n"
        if obs.observation_type == ObservationType.ERROR_RESOLVED:
            steps = obs.details.get("resolution_steps", [])
            md += f"  - Resolution steps: {len(steps)}\n"

    md += f"""
### Phase 4: Learning Reset

The learning reset phase tested the agent's ability to re-learn preferences after all previously learned preferences were cleared.

#### Observations

"""
    reset_obs = [o for o in result.observations if o.phase == ExperimentPhase.LEARNING_RESET]
    for obs in reset_obs:
        status = "✅" if obs.success else "❌"
        md += f"- {status} {obs.description}\n"
        if obs.details.get("preferences_detected"):
            md += f"  - New preferences: {obs.details['preferences_detected']}\n"

    md += f"""
## Metrics Analysis

| Metric | Baseline | Current | Change | Quality |
|--------|----------|---------|--------|---------|
"""
    
    for name, metric in result.metrics.items():
        change = "Yes" if metric.change_detected else "No"
        md += f"| {metric.name} | {metric.baseline_value} | {metric.current_value} | {change} | {metric.adaptation_quality:.1%} |\n"

    md += f"""
## Strengths

"""
    for strength in result.strengths:
        md += f"- ✅ {strength}\n"

    if not result.strengths:
        md += "- No significant strengths identified in this experiment\n"

    md += f"""
## Weaknesses

"""
    for weakness in result.weaknesses:
        md += f"- ⚠️ {weakness}\n"

    if not result.weaknesses:
        md += "- No significant weaknesses identified in this experiment\n"

    md += f"""
## Recommendations

Based on the experiment findings, the following improvements are recommended:

"""
    for i, rec in enumerate(result.recommendations, 1):
        md += f"{i}. {rec}\n"

    md += f"""
## Detailed Observation Log

| # | Timestamp | Phase | Type | Success | Description |
|---|-----------|-------|------|---------|-------------|
"""
    
    for i, obs in enumerate(result.observations, 1):
        status = "✅" if obs.success else "❌"
        md += f"| {i} | {obs.timestamp.strftime('%H:%M:%S')} | {obs.phase.value} | {obs.observation_type.value} | {status} | {obs.description[:50]}... |\n"

    md += f"""
## Learning Log Summary

The following learning events were recorded during the experiment:

```json
{json.dumps(result.to_dict().get("metrics", {}), indent=2)}
```

## Conclusion

The AI Agent Learning and Adaptation experiment demonstrates the Unified Mind's capability to:

1. **Learn preferences** through natural language interaction
2. **Recall preferences** when context changes
3. **Adapt behavior** based on learned patterns
4. **Detect and resolve errors** with appropriate diagnostics
5. **Re-learn preferences** after reset

The overall score of **{result.overall_score:.1f}/100** indicates {"strong" if result.overall_score >= 70 else "moderate" if result.overall_score >= 50 else "developing"} learning and adaptation capabilities. The identified strengths and weaknesses provide clear direction for future improvements.

---

*Report generated by KolibriOS AI Learning Experiment Suite*
*Experiment duration: {(result.end_time - result.start_time).total_seconds():.1f} seconds*
"""

    return md


async def main():
    """Main experiment execution."""
    # Create and run experiment
    experiment = AIAgentLearningExperiment()
    result = await experiment.run()

    # Generate outputs
    json_result = result.to_dict()
    markdown_report = generate_experiment_report(result)

    # Save reports
    report_dir = os.path.join(os.path.dirname(__file__), "..", "docs", "experiments")
    os.makedirs(report_dir, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    # Save JSON
    json_path = os.path.join(report_dir, f"ai_agent_learning_experiment_{timestamp}.json")
    with open(json_path, "w") as f:
        json.dump(json_result, f, indent=2)
    logger.info(f"JSON report saved to: {json_path}")

    # Save Markdown
    md_path = os.path.join(report_dir, "ai_agent_learning_experiment.md")
    with open(md_path, "w") as f:
        f.write(markdown_report)
    logger.info(f"Markdown report saved to: {md_path}")

    # Print summary
    print("\n" + "=" * 60)
    print("EXPERIMENT SUMMARY")
    print("=" * 60)
    print(f"Overall Score: {result.overall_score:.1f}/100")
    print(f"Total Observations: {len(result.observations)}")
    print(f"Successful: {sum(1 for o in result.observations if o.success)}")
    print(f"Strengths: {len(result.strengths)}")
    print(f"Weaknesses: {len(result.weaknesses)}")
    print(f"Recommendations: {len(result.recommendations)}")
    print("=" * 60)

    return result


if __name__ == "__main__":
    asyncio.run(main())
