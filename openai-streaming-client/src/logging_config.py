
import logging
import sys

def setup_logging(log_level='INFO', log_file=None):
    """
    Configure logging for the application.
    
    :param log_level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
    :param log_file: Optional file to log to. If None, logs to console.
    """
    # Convert log level string to logging constant
    log_level = getattr(logging, log_level.upper())

    # Configure basic logging
    logging_config = {
        "level": log_level,
        "format": "%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        "handlers": []
    }

    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(log_level)
    logging_config["handlers"].append(console_handler)

    # File handler if log_file is provided
    if log_file:
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(log_level)
        logging_config["handlers"].append(file_handler)

    logging.basicConfig(**logging_config)

    return logging.getLogger(__name__)

