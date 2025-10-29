//! WatsonX model definitions and constants
//!
//! Note: These constants represent commonly used models. For the most up-to-date
//! list of available models, use the `list_models()` method on `WatsonxClient`.

/// WatsonX model identifiers
pub mod models {
    // IBM Granite Models
    /// Granite 4.0 H Small model (default)
    pub const GRANITE_4_H_SMALL: &str = "ibm/granite-4-h-small";
    
    /// Granite 3.3 8B Instruct model
    pub const GRANITE_3_3_8B_INSTRUCT: &str = "ibm/granite-3-3-8b-instruct";
    
    /// Granite 3.3 8B Instruct NP (NorthPole optimized)
    pub const GRANITE_3_3_8B_INSTRUCT_NP: &str = "ibm/granite-3-3-8b-instruct-np";
    
    /// Granite 3.2 8B Instruct model
    pub const GRANITE_3_2_8B_INSTRUCT: &str = "ibm/granite-3-2-8b-instruct";
    
    /// Granite 3.2B Instruct model
    pub const GRANITE_3_2B_INSTRUCT: &str = "ibm/granite-3-2b-instruct";
    
    /// Granite 3.1 8B Base model
    pub const GRANITE_3_1_8B_BASE: &str = "ibm/granite-3-1-8b-base";
    
    /// Granite 3.8B Instruct model
    pub const GRANITE_3_8B_INSTRUCT: &str = "ibm/granite-3-8b-instruct";
    
    /// Granite 8B Code Instruct model
    pub const GRANITE_8B_CODE_INSTRUCT: &str = "ibm/granite-8b-code-instruct";
    
    /// Granite Guardian 3 8B model
    pub const GRANITE_GUARDIAN_3_8B: &str = "ibm/granite-guardian-3-8b";
    
    /// Granite Vision 3.2 2B model
    pub const GRANITE_VISION_3_2_2B: &str = "ibm/granite-vision-3-2-2b";
    
    // IBM Granite Embedding Models
    /// Granite Embedding 107M Multilingual model
    pub const GRANITE_EMBEDDING_107M_MULTILINGUAL: &str = "ibm/granite-embedding-107m-multilingual";
    
    /// Granite Embedding 278M Multilingual model
    pub const GRANITE_EMBEDDING_278M_MULTILINGUAL: &str = "ibm/granite-embedding-278m-multilingual";
    
    // IBM Granite Time Series Models
    /// Granite TTM 1024-96 R2 model
    pub const GRANITE_TTM_1024_96_R2: &str = "ibm/granite-ttm-1024-96-r2";
    
    /// Granite TTM 1536-96 R2 model
    pub const GRANITE_TTM_1536_96_R2: &str = "ibm/granite-ttm-1536-96-r2";
    
    /// Granite TTM 512-96 R2 model
    pub const GRANITE_TTM_512_96_R2: &str = "ibm/granite-ttm-512-96-r2";
    
    // IBM Slate Models
    /// Slate 125M English RTRVR model
    pub const SLATE_125M_ENGLISH_RTRVR: &str = "ibm/slate-125m-english-rtrvr";
    
    /// Slate 125M English RTRVR V2 model
    pub const SLATE_125M_ENGLISH_RTRVR_V2: &str = "ibm/slate-125m-english-rtrvr-v2";
    
    /// Slate 30M English RTRVR model
    pub const SLATE_30M_ENGLISH_RTRVR: &str = "ibm/slate-30m-english-rtrvr";
    
    /// Slate 30M English RTRVR V2 model
    pub const SLATE_30M_ENGLISH_RTRVR_V2: &str = "ibm/slate-30m-english-rtrvr-v2";
    
    // Meta Llama Models
    /// Llama 3.1 70B GPTQ model
    pub const LLAMA_3_1_70B_GPTQ: &str = "meta-llama/llama-3-1-70b-gptq";
    
    /// Llama 3.1 8B model
    pub const LLAMA_3_1_8B: &str = "meta-llama/llama-3-1-8b";
    
    /// Llama 3.2 11B Vision Instruct model
    pub const LLAMA_3_2_11B_VISION_INSTRUCT: &str = "meta-llama/llama-3-2-11b-vision-instruct";
    
    /// Llama 3.2 90B Vision Instruct model
    pub const LLAMA_3_2_90B_VISION_INSTRUCT: &str = "meta-llama/llama-3-2-90b-vision-instruct";
    
    /// Llama 3.3 70B Instruct model
    pub const LLAMA_3_3_70B_INSTRUCT: &str = "meta-llama/llama-3-3-70b-instruct";
    
    /// Llama 3.405B Instruct model
    pub const LLAMA_3_405B_INSTRUCT: &str = "meta-llama/llama-3-405b-instruct";
    
    /// Llama 4 Maverick 17B 128E Instruct FP8 model
    pub const LLAMA_4_MAVERICK_17B_128E_INSTRUCT_FP8: &str = "meta-llama/llama-4-maverick-17b-128e-instruct-fp8";
    
    /// Llama Guard 3 11B Vision model
    pub const LLAMA_GUARD_3_11B_VISION: &str = "meta-llama/llama-guard-3-11b-vision";
    
    // Mistral AI Models
    /// Mistral Medium 2505 model
    pub const MISTRAL_MEDIUM_2505: &str = "mistralai/mistral-medium-2505";
    
    /// Mistral Small 3.1 24B Instruct 2503 model
    pub const MISTRAL_SMALL_3_1_24B_INSTRUCT_2503: &str = "mistralai/mistral-small-3-1-24b-instruct-2503";
    
    // OpenAI Models
    /// GPT OSS 120B model
    pub const GPT_OSS_120B: &str = "openai/gpt-oss-120b";
    
    // Other Models
    /// Cross-encoder MS-Marco MiniLM L-12 V2 model
    pub const CROSS_ENCODER_MS_MARCO_MINILM_L_12_V2: &str = "cross-encoder/ms-marco-minilm-l-12-v2";
    
    /// IntFloat Multilingual E5 Large model
    pub const INTFLOAT_MULTILINGUAL_E5_LARGE: &str = "intfloat/multilingual-e5-large";
    
    /// Sentence Transformers All MiniLM L6 V2 model
    pub const SENTENCE_TRANSFORMERS_ALL_MINILM_L6_V2: &str = "sentence-transformers/all-minilm-l6-v2";
}

/// Default model to use
pub const DEFAULT_MODEL: &str = models::GRANITE_4_H_SMALL;

/// Maximum tokens supported by WatsonX models
pub const MAX_TOKENS_LIMIT: u32 = 131_072; // 128k tokens

/// Default maximum tokens for generation
pub const DEFAULT_MAX_TOKENS: u32 = 8192;

/// Conservative default for quick responses
pub const QUICK_RESPONSE_MAX_TOKENS: u32 = 2048;

/// Default timeout for API requests
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Default API version
pub const DEFAULT_API_VERSION: &str = "2023-05-29";

/// Default IAM URL for authentication
pub const DEFAULT_IAM_URL: &str = "iam.cloud.ibm.com";

/// Default API URL for WatsonX
pub const DEFAULT_API_URL: &str = "https://us-south.ml.cloud.ibm.com";
