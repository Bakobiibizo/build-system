
import openai
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class OpenAIClient:
    def __init__(self):
        self.api_key = os.getenv('OPENAI_API_KEY')
        if not self.api_key:
            raise ValueError("No OpenAI API key found. Please set OPENAI_API_KEY.")
        
        openai.api_key = self.api_key

    def stream_completion(self, prompt, model="gpt-3.5-turbo"):
        try:
            response = openai.ChatCompletion.create(
                model=model,
                messages=[{"role": "user", "content": prompt}],
                stream=True
            )
            
            for chunk in response:
                if chunk.choices[0].delta.get('content'):
                    yield chunk.choices[0].delta.content
        except Exception as e:
            print("Error in stream completion: {e}")
            yield None

def main():
    client = OpenAIClient()
    prompt = "Explain quantum computing in simple terms"
    
    print("Streaming response:")
    for chunk in client.stream_completion(prompt):
        if chunk:
            print(chunk, end='', flush=True)

if __name__ == "__main__":
    main()

