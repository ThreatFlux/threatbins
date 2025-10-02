#include <iostream>
#include <string>
#include <cstdlib>
#include <vector>
#include <algorithm>
#include <cctype>

#ifdef _WIN32
    #define PLATFORM_NAME "Windows"
    #define PING_COMMAND "ping -n 1 "
#elif __APPLE__
    #define PLATFORM_NAME "macOS"
    #define PING_COMMAND "ping -c 1 "
#elif __linux__
    #define PLATFORM_NAME "Linux"
    #define PING_COMMAND "ping -c 1 "
#else
    #define PLATFORM_NAME "Unknown"
    #define PING_COMMAND "ping -c 1 "
#endif

// Base64 encoding table
static const std::string base64_chars =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    "abcdefghijklmnopqrstuvwxyz"
    "0123456789+/";

// Function to encode string to base64
std::string base64_encode(const std::string& input) {
    std::string encoded;
    unsigned char char_array_3[3];
    unsigned char char_array_4[4];
    int i = 0;
    int j = 0;
    int in_len = input.size();

    while (in_len--) {
        char_array_3[i++] = input[j++];
        if (i == 3) {
            char_array_4[0] = (char_array_3[0] & 0xfc) >> 2;
            char_array_4[1] = ((char_array_3[0] & 0x03) << 4) + ((char_array_3[1] & 0xf0) >> 4);
            char_array_4[2] = ((char_array_3[1] & 0x0f) << 2) + ((char_array_3[2] & 0xc0) >> 6);
            char_array_4[3] = char_array_3[2] & 0x3f;

            for (i = 0; i < 4; i++) {
                encoded += base64_chars[char_array_4[i]];
            }
            i = 0;
        }
    }

    if (i) {
        for (int k = i; k < 3; k++) {
            char_array_3[k] = '\0';
        }

        char_array_4[0] = (char_array_3[0] & 0xfc) >> 2;
        char_array_4[1] = ((char_array_3[0] & 0x03) << 4) + ((char_array_3[1] & 0xf0) >> 4);
        char_array_4[2] = ((char_array_3[1] & 0x0f) << 2) + ((char_array_3[2] & 0xc0) >> 6);

        for (int k = 0; k < (i + 1); k++) {
            encoded += base64_chars[char_array_4[k]];
        }

        while ((i++ < 3)) {
            encoded += '=';
        }
    }

    return encoded;
}

// Function to check if a character is base64
static inline bool is_base64(unsigned char c) {
    return (isalnum(c) || (c == '+') || (c == '/'));
}

// Function to decode base64 to string
std::string base64_decode(const std::string& encoded_string) {
    int in_len = encoded_string.size();
    int i = 0;
    int j = 0;
    int in_ = 0;
    unsigned char char_array_4[4], char_array_3[3];
    std::string decoded;

    while (in_len-- && (encoded_string[in_] != '=') && is_base64(encoded_string[in_])) {
        char_array_4[i++] = encoded_string[in_]; in_++;
        if (i == 4) {
            for (i = 0; i < 4; i++) {
                char_array_4[i] = base64_chars.find(char_array_4[i]);
            }

            char_array_3[0] = (char_array_4[0] << 2) + ((char_array_4[1] & 0x30) >> 4);
            char_array_3[1] = ((char_array_4[1] & 0xf) << 4) + ((char_array_4[2] & 0x3c) >> 2);
            char_array_3[2] = ((char_array_4[2] & 0x3) << 6) + char_array_4[3];

            for (i = 0; i < 3; i++) {
                decoded += char_array_3[i];
            }
            i = 0;
        }
    }

    if (i) {
        for (j = 0; j < i; j++) {
            char_array_4[j] = base64_chars.find(char_array_4[j]);
        }

        char_array_3[0] = (char_array_4[0] << 2) + ((char_array_4[1] & 0x30) >> 4);
        char_array_3[1] = ((char_array_4[1] & 0xf) << 4) + ((char_array_4[2] & 0x3c) >> 2);

        for (j = 0; j < (i - 1); j++) {
            decoded += char_array_3[j];
        }
    }

    return decoded;
}

// Function to check if a string looks like base64
bool is_likely_base64(const std::string& str) {
    if (str.empty()) return false;

    // Check if string contains only base64 characters
    for (char c : str) {
        if (!is_base64(c) && c != '=') {
            return false;
        }
    }

    // Check if length is valid for base64 (must be multiple of 4 for proper base64)
    // But we'll be lenient and just check if it's reasonable
    return str.length() >= 4;
}

void ping_host(const std::string& ip) {
    std::string command = PING_COMMAND + ip;
    std::cout << "Pinging " << ip << " with 1 packet on " << PLATFORM_NAME << "..." << std::endl;
    int result = system(command.c_str());
    if (result == 0) {
        std::cout << "Ping successful!" << std::endl;
    } else {
        std::cout << "Ping failed!" << std::endl;
    }
}

void show_platform_info() {
    std::cout << "C++ SDK Platform Information:" << std::endl;
    std::cout << "  Platform: " << PLATFORM_NAME << std::endl;
    std::cout << "  Ping command: " << PING_COMMAND << "<ip>" << std::endl;

#ifdef _WIN32
    std::cout << "  Compiled for: Windows (x64)" << std::endl;
#elif __APPLE__
    std::cout << "  Compiled for: macOS (Universal)" << std::endl;
#elif __linux__
    std::cout << "  Compiled for: Linux (x64)" << std::endl;
#else
    std::cout << "  Compiled for: Unknown platform" << std::endl;
#endif
}

void show_help() {
    std::cout << "C++ SDK Usage (Cross-Platform):" << std::endl;
    std::cout << "  ./cpp-sdk                    - Print 'hello world'" << std::endl;
    std::cout << "  ./cpp-sdk <message>          - Print custom message (auto-decodes base64)" << std::endl;
    std::cout << "  ./cpp-sdk --ping             - Ping 1.1.1.1 (Cloudflare DNS)" << std::endl;
    std::cout << "  ./cpp-sdk --ping <ip>        - Ping specified IP address" << std::endl;
    std::cout << "  ./cpp-sdk --platform         - Show platform information" << std::endl;
    std::cout << "  ./cpp-sdk --encode <text>    - Encode text to base64" << std::endl;
    std::cout << "  ./cpp-sdk --decode <base64>  - Decode base64 to text" << std::endl;
    std::cout << "  ./cpp-sdk --auto-decode      - Toggle auto-decode for messages (default: on)" << std::endl;
    std::cout << "  ./cpp-sdk --help             - Show this help message" << std::endl;
    std::cout << std::endl;
    std::cout << "Features:" << std::endl;
    std::cout << "  - Automatic base64 detection and decoding for messages" << std::endl;
    std::cout << "  - Store sensitive data as base64 encoded strings" << std::endl;
    std::cout << "  - Cross-platform support: Windows, macOS, Linux" << std::endl;
}

int main(int argc, char* argv[]) {
    static bool auto_decode = true; // Enable auto-decode by default

    if (argc == 1) {
        // Default message stored as base64 at rest
        std::string encoded_default = "aGVsbG8gd29ybGQ="; // "hello world" in base64
        std::string message = base64_decode(encoded_default);
        std::cout << message << std::endl;
        return 0;
    }

    std::string arg1 = argv[1];

    if (arg1 == "--help" || arg1 == "-h") {
        show_help();
        return 0;
    }

    if (arg1 == "--platform") {
        show_platform_info();
        return 0;
    }

    if (arg1 == "--encode") {
        if (argc < 3) {
            std::cout << "Error: --encode requires text to encode" << std::endl;
            std::cout << "Usage: ./cpp-sdk --encode \"your text here\"" << std::endl;
            return 1;
        }
        std::string text_to_encode = argv[2];
        std::string encoded = base64_encode(text_to_encode);
        std::cout << "Original: " << text_to_encode << std::endl;
        std::cout << "Encoded:  " << encoded << std::endl;
        return 0;
    }

    if (arg1 == "--decode") {
        if (argc < 3) {
            std::cout << "Error: --decode requires base64 string to decode" << std::endl;
            std::cout << "Usage: ./cpp-sdk --decode \"base64string\"" << std::endl;
            return 1;
        }
        std::string text_to_decode = argv[2];
        try {
            std::string decoded = base64_decode(text_to_decode);
            std::cout << "Encoded:  " << text_to_decode << std::endl;
            std::cout << "Decoded:  " << decoded << std::endl;
        } catch (...) {
            std::cout << "Error: Invalid base64 string" << std::endl;
            return 1;
        }
        return 0;
    }

    if (arg1 == "--auto-decode") {
        auto_decode = !auto_decode;
        std::cout << "Auto-decode is now: " << (auto_decode ? "enabled" : "disabled") << std::endl;
        return 0;
    }

    if (arg1 == "--ping") {
        if (argc > 2) {
            // Ping custom IP
            ping_host(argv[2]);
        } else {
            // Ping default 1.1.1.1
            ping_host("1.1.1.1");
        }
        return 0;
    }

    // Default behavior: print the provided message with auto-decode if enabled
    std::string message = arg1;

    // Auto-decode base64 if it looks like base64 and auto-decode is enabled
    if (auto_decode && is_likely_base64(message)) {
        try {
            std::string decoded = base64_decode(message);
            std::cout << "[Base64 Detected - Decoded]: " << decoded << std::endl;
        } catch (...) {
            // If decode fails, just print the original
            std::cout << message << std::endl;
        }
    } else {
        std::cout << message << std::endl;
    }

    return 0;
}

