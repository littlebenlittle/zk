FROM alpine

RUN apk update
RUN apk add jq yj git
COPY ./spec/container/shellspec_install.sh /opt/shellspec_install.sh
RUN chmod +x /opt/shellspec_install.sh
RUN /opt/shellspec_install.sh --yes --prefix /usr/local
RUN rm -r /opt/shellspec_install.sh

ENTRYPOINT [ "/usr/local/bin/shellspec", "--sandbox" ]
