cmake_minimum_required(VERSION 3.16)
project(pp2p_core_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(pp2p_core_cpp STATIC
  src/pp2p_core_cpp.cpp
)

target_include_directories(pp2p_core_cpp PUBLIC
  ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_compile_definitions(pp2p_core_cpp PRIVATE
  PP2P_NATIVE_ROOT="${CMAKE_CURRENT_SOURCE_DIR}/../../native/pp2p_core"
)

if(UNIX AND NOT APPLE)
  target_link_libraries(pp2p_core_cpp PRIVATE dl)
endif()

add_executable(pp2p_core_cpp_example example.cpp)
target_link_libraries(pp2p_core_cpp_example PRIVATE pp2p_core_cpp)

add_custom_command(TARGET pp2p_core_cpp_example POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E make_directory $<TARGET_FILE_DIR:pp2p_core_cpp_example>/native/pp2p_core
  COMMAND ${CMAKE_COMMAND} -E copy_directory
          ${CMAKE_CURRENT_SOURCE_DIR}/../../native/pp2p_core
          $<TARGET_FILE_DIR:pp2p_core_cpp_example>/native/pp2p_core
)
