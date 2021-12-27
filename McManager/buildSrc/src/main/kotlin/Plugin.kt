import org.gradle.plugin.use.PluginDependenciesSpec
import org.gradle.plugin.use.PluginDependencySpec

inline val PluginDependenciesSpec.wire: PluginDependencySpec
    get() = id("com.squareup.wire")