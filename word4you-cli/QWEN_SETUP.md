# QWEN Setup Guide for Word4You

Word4You now supports QWEN (通义千问) as an alternative AI provider to Google Gemini. This guide will help you set up and use QWEN with Word4You.

## What is QWEN?

QWEN (通义千问) is Alibaba Cloud's large language model, available through the DashScope API. It provides high-quality text generation capabilities and is particularly good at Chinese language tasks.

## Getting Started

### 1. Get a QWEN API Key

1. Visit [Alibaba Cloud DashScope Console](https://dashscope.console.aliyun.com/)
2. Sign up or log in to your Alibaba Cloud account
3. Navigate to the API Keys section
4. Create a new API key
5. Copy the API key (it starts with `sk-`)

### 2. Configure Word4You

#### Option A: Interactive Configuration
```bash
word4you config
```
Follow the prompts to:
1. Select "qwen" as your AI provider
2. Enter your QWEN API key
3. Choose your preferred QWEN model (default: qwen-turbo)

#### Option B: Environment Variables
```bash
export WORD4YOU_AI_PROVIDER=qwen
export WORD4YOU_QWEN_API_KEY=your_qwen_api_key_here
export WORD4YOU_QWEN_MODEL_NAME=qwen-turbo
```

### 3. Use QWEN

#### Command Line Usage
```bash
# Use QWEN (default if configured)
word4you query beautiful

# Explicitly specify QWEN
word4you query beautiful --provider qwen

# Use raw output
word4you query beautiful --provider qwen --raw
```

#### Interactive Mode
```bash
word4you
# Enter words one by one, QWEN will be used automatically
```

## Available QWEN Models

- `qwen-turbo`: Fast and cost-effective (recommended)
- `qwen-plus`: Higher quality, slightly slower
- `qwen-max`: Highest quality, slower response

## Configuration File

Your configuration is stored in `~/.config/word4you/config.toml`:

```toml
ai_provider = "qwen"
qwen_api_key = "sk-your-api-key-here"
qwen_model_name = "qwen-turbo"
# ... other settings
```

## Troubleshooting

### Common Issues

1. **"QWEN API key not configured"**
   - Make sure you've set the API key in configuration
   - Run `word4you config` to set it up

2. **"QWEN API error: 401 Unauthorized"**
   - Check that your API key is correct
   - Ensure your Alibaba Cloud account has sufficient credits

3. **"QWEN API error: 429 Too Many Requests"**
   - You've exceeded the rate limit
   - Wait a moment and try again

### Testing Your Setup

```bash
# Test the API connection
word4you test
```

## Comparison: QWEN vs Gemini

| Feature | QWEN | Gemini |
|---------|------|--------|
| Chinese Support | Excellent | Good |
| English Support | Good | Excellent |
| Response Speed | Fast | Very Fast |
| Cost | Competitive | Competitive |
| API Stability | Good | Excellent |

## Switching Between Providers

You can easily switch between QWEN and Gemini:

```bash
# Use QWEN for one query
word4you query beautiful --provider qwen

# Use Gemini for another query
word4you query amazing --provider gemini

# Change default provider
word4you config
# Select your preferred provider
```

## Support

If you encounter issues with QWEN integration:

1. Check the [Alibaba Cloud DashScope documentation](https://help.aliyun.com/zh/dashscope/)
2. Verify your API key and account status
3. Check the Word4You GitHub issues for known problems

## Example Output

QWEN provides the same structured output format as Gemini:

```markdown
## beautiful

*/ˈbjuːtɪfʊl/*

> Pleasing to the senses or mind aesthetically.

**美丽的; 漂亮的; 美好的**

- She is a beautiful person inside and out.
- 她是一个内外都很美的人。

*"Beautiful" can describe both physical appearance and inner qualities.*
``` 