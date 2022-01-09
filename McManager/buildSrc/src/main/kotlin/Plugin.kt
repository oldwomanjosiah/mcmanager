import org.gradle.plugin.use.PluginDependenciesSpec
import org.gradle.plugin.use.PluginDependencySpec

/**
 * Add Wire Plugin
 */
inline val PluginDependenciesSpec.wire: PluginDependencySpec
    get() = id("com.squareup.wire")

/**
 * Add Compose Plugin
 */
inline val PluginDependenciesSpec.compose: PluginDependencySpec
    get() = id("org.jetbrains.compose")