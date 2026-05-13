"""Message handling decorators for SCP agents."""

from functools import wraps
from typing import Callable, Any, Dict, Optional, List
import inspect

from .types import MessageType


# Registry for handler functions
_HANDLER_REGISTRY: Dict[str, Dict[str, Any]] = {}


def on_message(
    msg_type: MessageType,
    priority: int = 0,
    description: str = "",
):
    """Decorator to register a method as a message handler.

    The decorated method will be automatically called when a message
    of the specified type is received.

    Args:
        msg_type: The SCP message type to handle.
        priority: Handler priority (lower = higher priority).
        description: Human-readable description of the handler.

    Example:
        ```python
        class MyBot(SCPAgent):
            @on_message(MessageType.TEXT)
            def handle_text(self, envelope, payload):
                print(f"Got text: {payload['body']}")

            @on_message(MessageType.AGENT_TASK, priority=10)
            def handle_task(self, envelope, payload):
                print(f"Got task: {payload['description']}")
        ```
    """
    def decorator(func: Callable) -> Callable:
        # Mark the function with metadata
        func._scp_message_type = msg_type
        func._scp_priority = priority
        func._scp_description = description

        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)
        return wrapper
    return decorator


def on_task(
    capability: Optional[str] = None,
    priority: int = 0,
):
    """Decorator for handling agent tasks.

    Args:
        capability: If set, only handle tasks matching this capability.
        priority: Handler priority (lower = higher priority).
    """
    def decorator(func: Callable) -> Callable:
        func._scp_message_type = MessageType.AGENT_TASK
        func._scp_capability = capability
        func._scp_priority = priority

        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)
        return wrapper
    return decorator


def on_approve(
    task_id: Optional[str] = None,
):
    """Decorator for handling task approval/rejection.

    Args:
        task_id: If set, only handle approval for this specific task.
    """
    def decorator(func: Callable) -> Callable:
        func._scp_message_type = MessageType.AGENT_APPROVE
        func._scp_task_id = task_id

        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)
        return wrapper
    return decorator


def get_handlers(obj: Any) -> List[Dict[str, Any]]:
    """Discover all decorated handler methods on an object.

    Returns a list of handler metadata dicts sorted by priority.
    """
    handlers = []
    for name, method in inspect.getmembers(obj, inspect.ismethod):
        msg_type = getattr(method, '_scp_message_type', None)
        if msg_type is not None:
            handlers.append({
                'name': name,
                'method': method,
                'msg_type': msg_type,
                'priority': getattr(method, '_scp_priority', 0),
                'capability': getattr(method, '_scp_capability', None),
                'description': getattr(method, '_scp_description', ''),
            })
    handlers.sort(key=lambda h: (h['priority'], h['name']))
    return handlers
