FROM libindy-test
ENV ANDROID_BUILD_FOLDER=/tmp/android_build
ENV ANDROID_SDK_ROOT=${ANDROID_SDK}
ENV ANDROID_HOME=${ANDROID_SDK}
ENV PATH=${PATH}:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/tools:${ANDROID_HOME}/tools/bin

ADD libindy/ci/android.prepare.sh .
ADD libindy/ci/setup.android.env.sh .
USER root
RUN chmod +x android.prepare.sh
RUN chown indy:indy android.prepare.sh
USER indy
RUN ["./android.prepare.sh", "subarch"]
