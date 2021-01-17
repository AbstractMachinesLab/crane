#!/usr/bin/ruby

require 'fileutils'

def erlang_library(path, name, deps)
  FileUtils.mkdir_p path
  File.write "./#{path}/Build.toml", """

# File generated for benchmarking
[[erlang_library]]
name = \"lib\"
deps = [#{ deps.join(", ") } ]
"""

  File.write "./#{path}/#{name}.erl", """

-module(#{name}).

"""

end

def workspace(name)
  FileUtils.mkdir_p name

  File.write "./#{name}/Workspace.toml", """

# File generated for benchmarking
[workspace]
name = \"#{name}\"

[toolchains.erlang]
archive_url = \"https://github.com/erlang/otp/archive/OTP-23.2.1.tar.gz\"
sha1 = \"b89dec86e41ce0cb696caab8420f652b339b0b25\"
prefix = \"otp-OTP-23.2.1\"
"""
end

workspace_name = "test_project"

workspace workspace_name

500.times do |i|
  deps = (0..(i-1)/5).map do |j| "\"//lib_#{j}:lib\"" end
  erlang_library "#{workspace_name}/lib_#{i}", "lib_#{i}", deps
end
