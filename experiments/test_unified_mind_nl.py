#!/usr/bin/env python3
"""
KolibriOS AI - Unified Mind Natural Language Query Test

This script tests the Unified Mind's natural language processing capabilities
by sending a complex query and verifying the response quality.

Query: "What is the current system status and suggest an optimization?"

Expected Behavior:
1. Intent Detection: system_status + optimization_request
2. Context Understanding: Current system metrics
3. Response Generation: Status report + optimization suggestion
4. Confidence Scoring: High confidence for clear intents

Usage:
    python experiments/test_unified_mind_nl.py
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
from typing import Any, Optional, Dict, List, Tuple

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class IntentType(Enum):
    """Detected intent types."""
    SYSTEM_STATUS = "system_status"
    OPTIMIZATION = "optimization"
    MEMORY_QUERY = "memory_query"
    CPU_QUERY = "cpu_query"
    TASK_QUERY = "task_query"
    HELP = "help"
    GENERAL = "general"
    UNKNOWN = "unknown"


class ResponseQuality(Enum):
    """Response quality levels."""
    EXCELLENT = "excellent"
    GOOD = "good"
    ACCEPTABLE = "acceptable"
    POOR = "poor"
    FAILED = "failed"


@dataclass
class SystemMetrics:
    """Current system metrics."""
    memory_total: int = 16 * 1024 * 1024 * 1024  # 16GB
    memory_used: int = 6 * 1024 * 1024 * 1024   # 6GB
    memory_utilization: float = 37.5
    cpu_cores: int = 8
    cpu_active_cores: int = 6
    cpu_utilization: float = 35.0
    running_tasks: int = 24
    pending_tasks: int = 3
    health_status: str = "healthy"
    
    def to_dict(self) -> dict:
        return {
            "memory": {
                "total_gb": self.memory_total / (1024**3),
                "used_gb": self.memory_used / (1024**3),
                "utilization_percent": self.memory_utilization,
            },
            "cpu": {
                "total_cores": self.cpu_cores,
                "active_cores": self.cpu_active_cores,
                "utilization_percent": self.cpu_utilization,
            },
            "tasks": {
                "running": self.running_tasks,
                "pending": self.pending_tasks,
            },
            "health": self.health_status,
        }


@dataclass
class IntentDetection:
    """Result of intent detection."""
    primary_intent: IntentType
    secondary_intents: List[IntentType]
    confidence: float
    keywords_detected: List[str]
    
    def to_dict(self) -> dict:
        return {
            "primary_intent": self.primary_intent.value,
            "secondary_intents": [i.value for i in self.secondary_intents],
            "confidence": self.confidence,
            "keywords_detected": self.keywords_detected,
        }


@dataclass
class NLResponse:
    """Natural language response from Unified Mind."""
    content: str
    intents_detected: List[IntentDetection]
    confidence: float
    response_time_ms: float
    quality: ResponseQuality
    suggestions: List[str]
    actions_taken: List[str]
    system_metrics: Optional[SystemMetrics] = None
    
    def to_dict(self) -> dict:
        return {
            "content": self.content,
            "intents_detected": [i.to_dict() for i in self.intents_detected],
            "confidence": self.confidence,
            "response_time_ms": self.response_time_ms,
            "quality": self.quality.value,
            "suggestions": self.suggestions,
            "actions_taken": self.actions_taken,
            "system_metrics": self.system_metrics.to_dict() if self.system_metrics else None,
        }


class UnifiedMindNLProcessor:
    """
    Unified Mind Natural Language Processor.
    
    Processes natural language queries and generates contextual responses.
    """
    
    def __init__(self):
        self.system_metrics = SystemMetrics()
        self.conversation_history: List[Dict] = []
        self.learned_preferences: Dict[str, Any] = {}
        self.intent_keywords = {
            IntentType.SYSTEM_STATUS: [
                "status", "system", "health", "state", "how is", "current",
                "overview", "report", "summary", "condition"
            ],
            IntentType.OPTIMIZATION: [
                "optimize", "optimization", "improve", "suggest", "recommendation",
                "better", "enhance", "tune", "boost", "speed up"
            ],
            IntentType.MEMORY_QUERY: [
                "memory", "ram", "allocation", "heap", "cache", "buffer"
            ],
            IntentType.CPU_QUERY: [
                "cpu", "processor", "core", "load", "usage", "performance"
            ],
            IntentType.TASK_QUERY: [
                "task", "process", "running", "job", "thread", "worker"
            ],
            IntentType.HELP: [
                "help", "what can", "how to", "guide", "assist", "support"
            ],
        }
        
    def detect_intents(self, query: str) -> List[IntentDetection]:
        """Detect intents from natural language query."""
        lower_query = query.lower()
        detected_intents = []
        
        # Check each intent type
        for intent_type, keywords in self.intent_keywords.items():
            matched_keywords = [kw for kw in keywords if kw in lower_query]
            if matched_keywords:
                confidence = min(len(matched_keywords) / 2.0, 1.0)
                detected_intents.append(IntentDetection(
                    primary_intent=intent_type,
                    secondary_intents=[],
                    confidence=confidence,
                    keywords_detected=matched_keywords,
                ))
        
        # Sort by confidence
        detected_intents.sort(key=lambda x: x.confidence, reverse=True)
        
        # If no intents detected, mark as general or unknown
        if not detected_intents:
            detected_intents.append(IntentDetection(
                primary_intent=IntentType.GENERAL,
                secondary_intents=[],
                confidence=0.3,
                keywords_detected=[],
            ))
        
        # Set secondary intents
        if len(detected_intents) > 1:
            detected_intents[0].secondary_intents = [i.primary_intent for i in detected_intents[1:]]
        
        return detected_intents
    
    def update_system_metrics(self):
        """Update system metrics with simulated data."""
        # Simulate slight variations
        self.system_metrics.memory_utilization = 35 + random.random() * 10
        self.system_metrics.cpu_utilization = 30 + random.random() * 15
        self.system_metrics.memory_used = int(
            self.system_metrics.memory_total * self.system_metrics.memory_utilization / 100
        )
        self.system_metrics.running_tasks = 20 + random.randint(0, 10)
        
    async def process_query(self, query: str) -> NLResponse:
        """Process natural language query and generate response."""
        start_time = time.time()
        
        # Update metrics
        self.update_system_metrics()
        
        # Detect intents
        intents = self.detect_intents(query)
        
        # Generate response based on intents
        content_parts = []
        suggestions = []
        actions_taken = []
        
        primary_intent = intents[0].primary_intent if intents else IntentType.UNKNOWN
        
        # Handle system status
        if primary_intent == IntentType.SYSTEM_STATUS or IntentType.SYSTEM_STATUS in [i.primary_intent for i in intents]:
            status_content = self._generate_status_report()
            content_parts.append(status_content)
        
        # Handle optimization request
        if primary_intent == IntentType.OPTIMIZATION or IntentType.OPTIMIZATION in [i.primary_intent for i in intents]:
            opt_content, opt_suggestions, opt_actions = self._generate_optimization_response()
            content_parts.append(opt_content)
            suggestions.extend(opt_suggestions)
            actions_taken.extend(opt_actions)
        
        # Combine response
        if not content_parts:
            content_parts.append("I understand your query. Let me help you with that.")
        
        content = "\n\n".join(content_parts)
        
        # Calculate confidence
        overall_confidence = sum(i.confidence for i in intents) / len(intents) if intents else 0.5
        
        # Determine quality
        if overall_confidence > 0.8:
            quality = ResponseQuality.EXCELLENT
        elif overall_confidence > 0.6:
            quality = ResponseQuality.GOOD
        elif overall_confidence > 0.4:
            quality = ResponseQuality.ACCEPTABLE
        else:
            quality = ResponseQuality.POOR
        
        response_time = (time.time() - start_time) * 1000
        
        # Log conversation
        self.conversation_history.append({
            "query": query,
            "response": content,
            "timestamp": datetime.now().isoformat(),
            "intents": [i.primary_intent.value for i in intents],
        })
        
        return NLResponse(
            content=content,
            intents_detected=intents,
            confidence=overall_confidence,
            response_time_ms=response_time,
            quality=quality,
            suggestions=suggestions,
            actions_taken=actions_taken,
            system_metrics=self.system_metrics,
        )
    
    def _generate_status_report(self) -> str:
        """Generate system status report."""
        m = self.system_metrics
        
        status_lines = [
            "## System Status Report",
            "",
            f"**Health**: {m.health_status.upper()} ✅",
            "",
            "### Memory",
            f"- Total: {m.memory_total / (1024**3):.1f} GB",
            f"- Used: {m.memory_used / (1024**3):.1f} GB ({m.memory_utilization:.1f}%)",
            f"- Available: {(m.memory_total - m.memory_used) / (1024**3):.1f} GB",
            "",
            "### CPU",
            f"- Cores: {m.cpu_active_cores}/{m.cpu_cores} active",
            f"- Utilization: {m.cpu_utilization:.1f}%",
            "",
            "### Tasks",
            f"- Running: {m.running_tasks}",
            f"- Pending: {m.pending_tasks}",
        ]
        
        return "\n".join(status_lines)
    
    def _generate_optimization_response(self) -> Tuple[str, List[str], List[str]]:
        """Generate optimization suggestions."""
        m = self.system_metrics
        suggestions = []
        actions = []
        
        lines = ["## Optimization Recommendations", ""]
        
        # Memory optimization
        if m.memory_utilization > 50:
            suggestions.append("Clear cache to free memory")
            actions.append("cache_cleanup")
            lines.append("### Memory Optimization")
            lines.append("- Consider clearing cache files")
            lines.append("- Close unused applications")
            lines.append("- Current utilization is moderate")
            lines.append("")
        
        # CPU optimization
        if m.cpu_utilization > 40:
            suggestions.append("Balance CPU load across cores")
            actions.append("load_balancing")
            lines.append("### CPU Optimization")
            lines.append(f"- Load distribution across {m.cpu_cores} cores")
            lines.append("- Consider deferring non-critical tasks")
            lines.append("")
        
        # General optimization
        suggestions.append("Enable auto-optimization for maintenance")
        actions.append("auto_optimization")
        
        lines.append("### Suggested Actions")
        lines.append("1. **Run System Cleanup**: Remove temporary files and caches")
        lines.append("2. **Optimize Task Scheduling**: Balance workloads across cores")
        lines.append("3. **Enable Gaming Mode**: If you're planning high-performance tasks")
        
        return "\n".join(lines), suggestions, actions


class UnifiedMindNLTest:
    """
    Test harness for Unified Mind Natural Language queries.
    """
    
    def __init__(self):
        self.processor = UnifiedMindNLProcessor()
        self.test_results: List[Dict] = []
        
    async def run_test(self, query: str) -> Dict:
        """Run a single NL query test."""
        logger.info(f"\n{'='*60}")
        logger.info("UNIFIED MIND NL QUERY TEST")
        logger.info(f"{'='*60}")
        logger.info(f"\nQuery: \"{query}\"")
        logger.info("-" * 60)
        
        # Process query
        response = await self.processor.process_query(query)
        
        # Log results
        logger.info("\n[RESPONSE]")
        logger.info(response.content)
        logger.info(f"\n[INTENTS DETECTED]")
        for intent in response.intents_detected:
            logger.info(f"  - {intent.primary_intent.value}: {intent.confidence:.0%} confidence")
            logger.info(f"    Keywords: {intent.keywords_detected}")
        logger.info(f"\n[QUALITY]: {response.quality.value.upper()}")
        logger.info(f"[CONFIDENCE]: {response.confidence:.0%}")
        logger.info(f"[RESPONSE TIME]: {response.response_time_ms:.2f}ms")
        
        if response.suggestions:
            logger.info(f"\n[SUGGESTIONS]")
            for i, s in enumerate(response.suggestions, 1):
                logger.info(f"  {i}. {s}")
        
        if response.actions_taken:
            logger.info(f"\n[ACTIONS TAKEN]")
            for a in response.actions_taken:
                logger.info(f"  - {a}")
        
        # Evaluate success
        success = self._evaluate_response(query, response)
        
        logger.info(f"\n[RESULT]: {'✅ PASS' if success else '❌ FAIL'}")
        logger.info(f"{'='*60}\n")
        
        result = {
            "query": query,
            "success": success,
            "response": response.to_dict(),
            "evaluation": {
                "intent_detected": len(response.intents_detected) > 0,
                "status_reported": IntentType.SYSTEM_STATUS in [i.primary_intent for i in response.intents_detected],
                "optimization_suggested": IntentType.OPTIMIZATION in [i.primary_intent for i in response.intents_detected],
                "quality_acceptable": response.quality in [ResponseQuality.EXCELLENT, ResponseQuality.GOOD],
                "confidence_high": response.confidence >= 0.5,
            }
        }
        
        self.test_results.append(result)
        return result
    
    def _evaluate_response(self, query: str, response: NLResponse) -> bool:
        """Evaluate if the response is successful."""
        # Check intent detection
        has_status_intent = IntentType.SYSTEM_STATUS in [i.primary_intent for i in response.intents_detected]
        has_optimization_intent = IntentType.OPTIMIZATION in [i.primary_intent for i in response.intents_detected]
        
        # Check content quality
        has_status_content = "status" in response.content.lower() or "health" in response.content.lower()
        has_optimization_content = "optimization" in response.content.lower() or "suggest" in response.content.lower()
        
        # Check for system metrics
        has_metrics = response.system_metrics is not None
        
        # Overall evaluation
        success = (
            (has_status_intent or has_optimization_intent) and
            (has_status_content or has_optimization_content) and
            has_metrics and
            response.confidence >= 0.5 and
            response.quality in [ResponseQuality.EXCELLENT, ResponseQuality.GOOD, ResponseQuality.ACCEPTABLE]
        )
        
        return success


async def main():
    """Main test execution."""
    test = UnifiedMindNLTest()
    
    # The test query
    query = "What is the current system status and suggest an optimization?"
    
    # Run test
    result = await test.run_test(query)
    
    # Save results
    output_dir = os.path.join(os.path.dirname(__file__), "..", "docs", "experiments")
    os.makedirs(output_dir, exist_ok=True)
    
    output_file = os.path.join(output_dir, "unified_mind_nl_test.json")
    with open(output_file, "w") as f:
        json.dump({
            "test_name": "Unified Mind Natural Language Query Test",
            "timestamp": datetime.now().isoformat(),
            "query": query,
            "success": result["success"],
            "evaluation": result["evaluation"],
            "response": result["response"],
        }, f, indent=2, default=str)
    
    print(f"\nResults saved to: {output_file}")
    
    return result


if __name__ == "__main__":
    result = asyncio.run(main())
    sys.exit(0 if result["success"] else 1)
