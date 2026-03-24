cmake_minimum_required(VERSION 3.16)
project(pp2p_core_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(pp2p_core_cpp STATIC
  src/pp2p_core_cpp.cpp
)

target_include_directories(pp2p_core_cpp PUBLIC
  ${CMAKE_CURRENT_SOURCE_DIR}/include
  ${CMAKE_CURRENT_SOURCE_DIR}/../../include
)

if(WIN32)
  target_link_libraries(pp2p_core_cpp PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../../dist/pp2p_core/windows-x64/pp2p_core.lib
  )
elseif(APPLE)
  target_link_libraries(pp2p_core_cpp PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../../dist/pp2p_core/macos/libpp2p_core.dylib
  )
else()
  target_link_libraries(pp2p_core_cpp PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../../dist/pp2p_core/linux-x64/libpp2p_core.so
  )
endif()

add_executable(pp2p_core_cpp_example example.cpp)
target_link_libraries(pp2p_core_cpp_example PRIVATE pp2p_core_cpp)
