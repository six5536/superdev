pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "{{superdev:project-pascal}}"
include(":app")
include(":native-debug-server")
project(":native-debug-server").projectDir = file("../../libs/native-debug-server-android")
