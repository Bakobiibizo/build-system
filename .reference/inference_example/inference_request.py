import json
import os
import sys
import traceback
import signal
from contextlib import contextmanager
from openai import OpenAI
from dotenv import load_dotenv

load_dotenv()

class TimeoutException(Exception):
    pass

@contextmanager
def timeout(seconds):
    def _handle_timeout(signum, frame):
        raise TimeoutException(f"Timeout after {seconds} seconds")
    
    signal.signal(signal.SIGALRM, _handle_timeout)
    signal.alarm(seconds)
    try:
        yield
    finally:
        signal.alarm(0)

def create_completion(client: OpenAI, model: str, prompt: str, temperature: float=0.6, stream: bool=True, timeout_seconds: int=60) -> str:
    try:
        print(f"\n--- Streaming Response from {model} ---")
        print(f"Base URL: {client._base_url}")
        sys.stdout.flush()

        full_response = ""
        with open(".reference/inference_example/response.txt", "w", encoding="utf-8") as response_file:
            try:
                with timeout(timeout_seconds):
                    completion = client.chat.completions.create(
                        model=model,
                        messages=[{"role": "user", "content": prompt}],
                        temperature=temperature,
                        stream=stream
                    )
                    
                    for chunk in completion:
                        try:
                            if chunk.choices and chunk.choices[0].delta and chunk.choices[0].delta.content:
                                chunk_content = chunk.choices[0].delta.content
                                full_response += chunk_content
                                response_file.write(chunk_content)
                                response_file.flush()
                                print(chunk_content, end='', flush=True)
                        except Exception as chunk_error:
                            print(f"\nError processing chunk: {chunk_error}")
                            traceback.print_exc()
                    
                    print("\n--- Stream Complete ---")
                    sys.stdout.flush()
            
            except TimeoutException:
                print(f"\n--- Timeout after {timeout_seconds} seconds ---")
                sys.stdout.flush()
                return None
        
        return full_response
    except Exception as e:
        print(f"\nFull Error Details:")
        print(f"Error Type: {type(e)}")
        print(f"Error Message: {str(e)}")
        traceback.print_exc()
        sys.stdout.flush()
        return None

def get_prompt() -> str:
    try:
        with open(".reference/inference_example/prompt.txt", "r", encoding="utf-8") as f:
            return f.read().strip()
    except Exception as e:
        print(f"Error reading prompt: {e}")
        traceback.print_exc()
        return None

def main():
    try:
        # Print environment variables for debugging
        print("Environment Variables:")
        print(f"API Key: {os.getenv('INFERENCE_API_KEY')[:4]}...")
        print(f"Base URL: {os.getenv('INFERENCE_API_BASE_URL')}")
        print(f"Model: {os.getenv('INFERENCE_API_MODEL')}")
        sys.stdout.flush()

        openai = OpenAI(
            api_key=os.getenv("INFERENCE_API_KEY"),
            base_url=os.getenv("INFERENCE_API_BASE_URL"),
        )
        model = os.getenv("INFERENCE_API_MODEL")
        prompt = get_prompt()
        
        if prompt is None:
            print("Error: Prompt is empty")
            return
        
        print(f"\nPrompt Length: {len(prompt)} characters")
        sys.stdout.flush()

        completion = create_completion(openai, model, prompt, timeout_seconds=900)
        
        if completion is None:
            print("Error: No completion generated")
            return
        
    except Exception as e:
        print(f"Unexpected error: {e}")
        traceback.print_exc()
        sys.stdout.flush()

if __name__ == "__main__":
    main()
