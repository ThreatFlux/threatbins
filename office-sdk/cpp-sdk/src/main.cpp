#include <iostream>
#include <string>

int main(int argc, char* argv[]) {
    std::string input = "hello world";

    // If command line argument provided, use it instead
    if (argc > 1) {
        input = argv[1];
    }

    std::cout << input << std::endl;

    return 0;
}