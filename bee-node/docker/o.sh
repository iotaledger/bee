DASHBOARD_PORT=`cat ./config.toml | grep "port" | tail -1 | sed 's/["port=]//g' | tr -d '[[:space:]]'`
MQTT_PORT=`cat ./config.toml | grep "address"| tail -1 | sed 's/["address=tcp://loclhot:]//g' | tr -d '[[:space:]]'`
BINDING_PORT=`cat ./config.toml | grep "binding_port" | tail -1 | sed 's/["binding_port=]//g' | tr -d '[[:space:]]'`
BIND_ADDRESS_PORT=`cat ./config.toml | grep "bind_address"| head -1 | sed 's#.*/\([^:]*\).*#\1#' | sed 's/["]//g' | tr -d '[[:space:]]'`

echo ${DASHBOARD_PORT}
# echo $MQTT_PORT
# echo $BINDING_PORT
# echo $BIND_ADDRESS_PORT
