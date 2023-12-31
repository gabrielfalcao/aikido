class {{ formula.name }} < Formula
  desc "{{ formula.desc }}"
  homepage "{{ formula.homepage }}"
  # ...TK
  # url "https://github.com/gabrielfalcao/aikido/archive/v0.0.1.tar.gz"
  # sha256 "30b6256bea0143caebd08256e0a605280afbbc5eef7ce692f84621eb232a9b31"
  license any_of: [{{ formula.licenses }}]
  head "https://github.com/gabrielfalcao/aikido.git", branch: "main"

  depends_on macos: :ventura

  depends_on formula: "ack"
  depends_on formula: "adwaita-icon-theme"
  depends_on formula: "aften"
  depends_on formula: "aom"
  depends_on formula: "atk"
  depends_on formula: "autoconf"
  depends_on formula: "automake"
  depends_on formula: "bash"
  depends_on formula: "bash-completion"
  depends_on formula: "bash-completion@2"
  depends_on formula: "bash-git-prompt"
  depends_on formula: "berkeley-db"
  depends_on formula: "boost"
  depends_on formula: "brotli"
  depends_on formula: "ca-certificates"
  depends_on formula: "cairo"
  depends_on formula: "carthage"
  depends_on formula: "coreutils"
  depends_on formula: "cppzmq"
  depends_on formula: "curl"
  depends_on formula: "e2fsprogs"
  depends_on formula: "elixir"
  depends_on formula: "erlang"
  depends_on formula: "exiftool"
  depends_on formula: "fftw"
  depends_on formula: "figlet"
  depends_on formula: "flac"
  depends_on formula: "fmt"
  depends_on formula: "fontconfig"
  depends_on formula: "freetype"
  depends_on formula: "fribidi"
  depends_on formula: "gawk"
  depends_on formula: "gcc"
  depends_on formula: "gdbm"
  depends_on formula: "gdk-pixbuf"
  depends_on formula: "gettext"
  depends_on formula: "ghostscript"
  depends_on formula: "giflib"
  depends_on formula: "git"
  depends_on formula: "glib"
  depends_on formula: "gmp"
  depends_on formula: "gnu-sed"
  depends_on formula: "gnu-tar"
  depends_on formula: "gnu-time"
  depends_on formula: "gnu-which"
  depends_on formula: "gnupg"
  depends_on formula: "gnuradio"
  depends_on formula: "gnutls"
  depends_on formula: "go"
  depends_on formula: "gobject-introspection"
  depends_on formula: "graphite2"
  depends_on formula: "gsettings-desktop-schemas"
  depends_on formula: "gsl"
  depends_on formula: "gtk+3"
  depends_on formula: "harfbuzz"
  depends_on formula: "hicolor-icon-theme"
  depends_on formula: "highway"
  depends_on formula: "hwloc"
  depends_on formula: "icu4c"
  depends_on formula: "imagemagick"
  depends_on formula: "imath"
  depends_on formula: "isl"
  depends_on formula: "jack"
  depends_on formula: "jasper"
  depends_on formula: "jbig2dec"
  depends_on formula: "jpeg-turbo"
  depends_on formula: "jpeg-xl"
  depends_on formula: "jq"
  depends_on formula: "krb5"
  depends_on formula: "lame"
  depends_on formula: "libassuan"
  depends_on formula: "libde265"
  depends_on formula: "libepoxy"
  depends_on formula: "libevent"
  depends_on formula: "libgcrypt"
  depends_on formula: "libgpg-error"
  depends_on formula: "libheif"
  depends_on formula: "libidn"
  depends_on formula: "libidn2"
  depends_on formula: "libksba"
  depends_on formula: "liblqr"
  depends_on formula: "libmpc"
  depends_on formula: "libnghttp2"
  depends_on formula: "libogg"
  depends_on formula: "libomp"
  depends_on formula: "libpng"
  depends_on formula: "libraw"
  depends_on formula: "librsvg"
  depends_on formula: "librtlsdr"
  depends_on formula: "libsamplerate"
  depends_on formula: "libsigsegv"
  depends_on formula: "libsndfile"
  depends_on formula: "libsodium"
  depends_on formula: "libssh2"
  depends_on formula: "libsvg"
  depends_on formula: "libsvg-cairo"
  depends_on formula: "libtasn1"
  depends_on formula: "libtiff"
  depends_on formula: "libtool"
  depends_on formula: "libunistring"
  depends_on formula: "libusb"
  depends_on formula: "libvmaf"
  depends_on formula: "libvorbis"
  depends_on formula: "libx11"
  depends_on formula: "libxau"
  depends_on formula: "libxcb"
  depends_on formula: "libxdmcp"
  depends_on formula: "libxext"
  depends_on formula: "libxrender"
  depends_on formula: "libyaml"
  depends_on formula: "little-cms2"
  depends_on formula: "log4cpp"
  depends_on formula: "lua"
  depends_on formula: "luarocks"
  depends_on formula: "lz4"
  depends_on formula: "lzo"
  depends_on formula: "m4"
  depends_on formula: "mpdecimal"
  depends_on formula: "mpfr"
  depends_on formula: "mpg123"
  depends_on formula: "nettle"
  depends_on formula: "npth"
  depends_on formula: "numpy"
  depends_on formula: "nvm"
  depends_on formula: "oniguruma"
  depends_on formula: "open-mpi"
  depends_on formula: "openblas"
  depends_on formula: "openexr"
  depends_on formula: "openjpeg"
  depends_on formula: "openldap"
  depends_on formula: "openssl@3"
  depends_on formula: "opus"
  depends_on formula: "orc"
  depends_on formula: "p11-kit"
  depends_on formula: "pango"
  depends_on formula: "pcre2"
  depends_on formula: "pinentry"
  depends_on formula: "pixman"
  depends_on formula: "pkg-config"
  depends_on formula: "portaudio"
  depends_on formula: "postgresql@14"
  depends_on formula: "py3cairo"
  depends_on formula: "pygments"
  depends_on formula: "pygobject3"
  depends_on formula: "pyqt@5"
  depends_on formula: "python@3.11"
  depends_on formula: "pyyaml"
  depends_on formula: "qt@5"
  depends_on formula: "qwt-qt5"
  depends_on formula: "rabbitmq"
  depends_on formula: "readline"
  depends_on formula: "redis"
  depends_on formula: "rtmpdump"
  depends_on formula: "ruby"
  depends_on formula: "shared-mime-info"
  depends_on formula: "six"
  depends_on formula: "soapyrtlsdr"
  depends_on formula: "soapysdr"
  depends_on formula: "spdlog"
  depends_on formula: "sqlite"
  depends_on formula: "svg2png"
  depends_on formula: "swift"
  depends_on formula: "switchaudio-osx"
  depends_on formula: "tree"
  depends_on formula: "uhd"
  depends_on formula: "unbound"
  depends_on formula: "unixodbc"
  depends_on formula: "volk"
  depends_on formula: "webp"
  depends_on formula: "wget"
  depends_on formula: "wxwidgets"
  depends_on formula: "x265"
  depends_on formula: "xorgproto"
  depends_on formula: "xz"
  depends_on formula: "zeromq"
  depends_on formula: "zstd"

  depends_on cask: "bitwarden"
  depends_on cask: "blackhole-2ch"
  depends_on cask: "brave-browser"
  depends_on cask: "emacs"
  depends_on cask: "google-chrome"
  depends_on cask: "iterm2"
  depends_on cask: "keepassxc"
  depends_on cask: "macfuse"
  depends_on cask: "obsidian"
  depends_on cask: "phoenix"
  depends_on cask: "spotify"
  depends_on cask: "vivaldi"


  def install
    system "bash", "./install.sh"
  end
end
