#!/bin/sh

#
# Gradle wrapper script for POSIX systems
#

# Resolve the project directory
APP_HOME=$( cd -P "$( dirname "$0" )" > /dev/null && pwd )

# Determine Java command
if [ -n "$JAVA_HOME" ] ; then
    JAVACMD="$JAVA_HOME/bin/java"
else
    JAVACMD="java"
fi

# Check Java availability
if ! command -v "$JAVACMD" >/dev/null 2>&1; then
    echo "ERROR: JAVA_HOME is not set and 'java' command not found." >&2
    exit 1
fi

# Run Gradle wrapper
exec "$JAVACMD" \
    -classpath "$APP_HOME/gradle/wrapper/gradle-wrapper.jar" \
    org.gradle.wrapper.GradleWrapperMain \
    "$@"
