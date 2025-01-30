
import time
import logging

class RetryableError(Exception):
    """Custom exception for errors that can be retried."""
    pass

def retry_with_backoff(
    func, 
    max_retries=3, 
    base_delay=1, 
    max_delay=60
):
    """
    Retry a function with exponential backoff.
    
    :param func: Function to retry
    :param max_retries: Maximum number of retry attempts
    :param base_delay: Initial delay between retries
    :param max_delay: Maximum delay between retries
    :return: Result of the function
    """
    retries = 0
    delay = base_delay

    while retries < max_retries:
        try:
            return func()
        except RetryableError as error:
            logging.warning("Attempt {retries + 1} failed: {error}")
            time.sleep(delay)
            delay = min(delay * 2, max_delay)
            retries += 1

    raise Exception("Function failed after {max_retries} attempts")

