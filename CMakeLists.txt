cmake_minimum_required(VERSION 3.28.1)

set(CMAKE_C_STANDARD 23)

project(terminal-engine VERSION 1.0 LANGUAGES C)

file(GLOB_RECURSE src CMAKE_CONFIGURE_DEPENDS "src/*.c")

add_executable(terminal-engine ${src})

target_include_directories(terminal-engine PRIVATE "include")
